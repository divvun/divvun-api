use actix_web::{AsyncResponder, HttpResponse, Json, Path, State};

use actix::prelude::*;
use futures::future::{result, Future};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

use crate::{ApiError, State as AppState};
use serde_derive::{Deserialize, Serialize};

use std::process::{Child, Command, Stdio};

use regex::Regex;
use std::collections::BTreeMap;
use std::io::Error;

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

#[derive(Deserialize)]
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

/// Async handler
// pub fn post_gramcheck((body, language, state): (Json<GramcheckRequest>, Path<String>, State<AppState>)) -> Box<Future<Item=HttpResponse, Error=actix_web::Error>> {
pub fn post_gramcheck(
    language: Path<String>,
    state: State<AppState>,
    body: Json<GramcheckRequest>,
) -> Box<Future<Item = HttpResponse, Error = actix_web::Error>> {
    let gramcheck = match state.gramcheckers.get(&*language) {
        Some(s) => s,
        None => {
            return result(Ok(HttpResponse::InternalServerError().into())).responder();
        }
    };

    gramcheck
        .send(body.0)
        .from_err()
        .and_then(|res| match res {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn get_gramcheck_preferences(
    language: Path<String>,
    state: State<AppState>,
) -> Box<Future<Item = HttpResponse, Error = actix_web::Error>> {
    let error_tags = match state.gramcheck_preferences.get(&*language) {
        Some(s) => s,
        None => {
            return result(Ok(HttpResponse::InternalServerError().into())).responder();
        }
    };

    result(Ok(HttpResponse::Ok().json(GramcheckPreferencesResponse {
        error_tags: error_tags.to_owned(),
    })))
    .responder()
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_foo() {
        let some_data =
            json!({"errs":[["heno",0,4,"typo","Čállinmeattáhus",[]]],"text":"heno."});
    }
}
