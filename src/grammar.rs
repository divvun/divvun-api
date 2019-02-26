use actix_web::{Json, Path, HttpResponse, State, AsyncResponder};

use futures::future::{Future, result};
use actix::prelude::*;
use std::io::Write;
use std::io::BufRead;
use std::io::BufReader;

use serde_derive::{Deserialize, Serialize};
use crate::{ApiError, State as AppState};

use std::process::{Command, Stdio, Child};

pub struct GramcheckExecutor(pub Child);

impl GramcheckExecutor {
    pub fn new(data_file_path: &str) -> Result<Self, std::io::Error> {
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
    text: String
}

impl Message for GramcheckRequest {
    type Result = Result<GramcheckOutput, ApiError>;
}

impl Handler<GramcheckRequest> for GramcheckExecutor {
    type Result = Result<GramcheckOutput, ApiError>;

    fn handle(&mut self, msg: GramcheckRequest, _: &mut Self::Context) -> Self::Result {
        let stdin = self.0.stdin.as_mut().expect("Failed to open stdin");
        let mut stdout = BufReader::new(self.0.stdout.as_mut().unwrap());

        stdin.write_all(msg.text.as_bytes()).unwrap();
        stdin.write("\n".as_bytes()).unwrap();

        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();

        let result: GramcheckOutput = serde_json::from_str(&line).unwrap();
        Ok(result)
    }
}

#[derive(Deserialize, Serialize)]
pub struct GramcheckErrResponse {
    error_text: String,
    start_index: u32,
    end_index: u32,
    error_code: String,
    description: String,
    suggestions: Vec<String>
}

#[derive(Deserialize, Serialize)]
pub struct GramcheckOutput {
    text: String,
    errs: Vec<GramcheckErrResponse>
}

/// Async handler
// pub fn post_gramcheck((body, language, state): (Json<GramcheckRequest>, Path<String>, State<AppState>)) -> Box<Future<Item=HttpResponse, Error=actix_web::Error>> {
pub fn post_gramcheck(language: Path<String>, state: State<AppState>, body: Json<GramcheckRequest>) -> Box<Future<Item=HttpResponse, Error=actix_web::Error>> {
    let gramcheck = match state.gramcheckers.get(&*language) {
        Some(s) => s,
        None => {
            return result(Ok(HttpResponse::InternalServerError().into())).responder();
        }
    };

    gramcheck.send(body.0).from_err().and_then(|res| {
        match res {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into())
        }
    }).responder()
}


#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_foo() {
        let some_data = json!({"errs":[["heno",0,4,"typo","Čállinmeattáhus",[]]],"text":"heno."});


    }
}