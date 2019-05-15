use std::collections::BTreeMap;
use std::process::{Child, Command, Stdio};
use std::io::{Error, BufRead, BufReader, Write};

use hashbrown::HashMap;
use serde_derive::{Deserialize, Serialize};
use regex::Regex;

use actix_web::{HttpResponse, web};
use actix::prelude::*;
use futures::future::{result, err, Future};

use crate::server::{ApiError as OldApiError, State as AppState};
use crate::state::{GrammarSuggestions, ApiError, State};

pub fn list_preferences(data_file_path: &str) -> Result<BTreeMap<String, String>, Error> {
    let mut process = Command::new("divvun-checker")
        .arg("-a")
        .arg(data_file_path)
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = BufReader::new(process.stdout.as_mut().expect("stdout to not be dead"));

    let regex = Regex::new(r"- \[.\] ([^\s+]+)\s+(.+)$").expect("valid regex");

    let categories: BTreeMap<String, String> = stdout
        .lines()
        .into_iter()
        .map(|l| l.unwrap())
        .skip_while(|l| l != "==== Toggles: ====")
        .skip(1)
        // temporary solution
        .skip_while(|l| l != "==== Toggles: ====")
        .skip(1)
        .map(|l| {
            regex
                .captures(&l)
                .map(|c| (c[1].to_owned(), c[2].to_owned()))
        })
        .take_while(|m| m.is_some())
        .map(|m| m.unwrap())
        .filter(|m| m.0 != "[regex]")
        .collect();

    Ok(categories)
}

pub struct GramcheckExecutor(pub Child);

impl GramcheckExecutor {
    pub fn new(data_file_path: &str) -> Result<Self, Error> {
        let process = Command::new("divvun-checker")
            .arg("-a")
            .arg(data_file_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Self(process))
    }
}

impl Actor for GramcheckExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Deserialize)]
pub struct GramcheckRequest {
    text: String,
}

impl Message for GramcheckRequest {
    type Result = Result<GramcheckOutput, ApiError>;
}

impl Handler<GramcheckRequest> for GramcheckExecutor {
    type Result = Result<GramcheckOutput, ApiError>;

    fn handle(&mut self, msg: GramcheckRequest, _: &mut Self::Context) -> Self::Result {
        let stdin = self.0.stdin.as_mut().expect("Failed to open stdin");
        let mut stdout = BufReader::new(self.0.stdout.as_mut().expect("stdout to not be dead"));

        let cleaned_msg = msg
            .text
            .split("\n")
            .next()
            .expect("string from newline split");

        stdin.write_all(cleaned_msg.as_bytes()).expect("write all");
        stdin.write("\n".as_bytes()).expect("write nl");

        let mut line = String::new();
        stdout.read_line(&mut line).expect("read a line");

        match serde_json::from_str(&line) {
            Ok(result) => return Ok(result),
            Err(err) => {
                return Err(ApiError {
                    message: format!("error: {:?}, line: '{}'", &err, &line),
                })
            }
        };
    }
}

#[derive(Deserialize, Serialize)]
pub struct GramcheckPreferencesResponse {
    error_tags: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
pub struct GramcheckErrResponse {
    error_text: String,
    start_index: u32,
    end_index: u32,
    error_code: String,
    description: String,
    suggestions: Vec<String>,
    title: String,
}

#[derive(Deserialize, Serialize)]
pub struct GramcheckOutput {
    text: String,
    errs: Vec<GramcheckErrResponse>,
}

pub fn get_gramcheck_preferences(
    language: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let error_tags = match state.gramcheck_preferences.get(&*language) {
        Some(s) => s,
        None => {
            return result(Ok(HttpResponse::InternalServerError().into()));
        }
    };

    result(Ok(HttpResponse::Ok().json(GramcheckPreferencesResponse {
        error_tags: error_tags.to_owned(),
    })))
}

pub struct AsyncGramchecker {
    pub gramcheckers: HashMap<String, Addr<GramcheckExecutor>>,
}

impl GrammarSuggestions for AsyncGramchecker {
    fn grammar_suggestions(&self, message: GramcheckRequest, language: &str)
-> Box<Future<Item=Result<GramcheckOutput, ApiError>, Error=ApiError>> {

        let gramchecker = match self.gramcheckers.get(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: "No grammar checker available for that language".to_owned()
                }));
            }
        };

        Box::new(
            gramchecker
                .send(message)
                .map_err(|err|
                    ApiError { message: format!("Something failed in the message delivery process: {}", err) }
        ))
    }
}

pub fn gramchecker_handler(
    body: web::Json<GramcheckRequest>,
    path: web::Path<String>,
    state: web::Data<State>)
-> impl Future<Item=HttpResponse, Error=actix_web::Error> {

    let grammar_suggestions = &state.language_functions.grammar_suggesgions;

    grammar_suggestions.grammar_suggestions(body.0, &path)
        .from_err()
        .and_then(|res| match res {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
}

#[cfg(test)]
mod test {
    use serde_json::json;

    #[test]
    fn test_foo() {
        let _some_data =
            json!({"errs":[["heno",0,4,"typo","Čállinmeattáhus",[]]],"text":"heno."});
    }
}
