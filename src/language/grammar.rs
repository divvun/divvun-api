use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Error, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

use actix::prelude::*;
use futures::future::{err, ok, Future};
use hashbrown::HashMap;
use log::{debug, error, info, warn};
use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::server::state::{LanguageSuggestions, UnhoistFutureExt};

pub struct GramcheckExecutor {
    pub child: Child,
    pub path: String,
    pub language: String,
    pub terminated: bool,
}

impl GramcheckExecutor {
    pub fn new(data_file_path: &str, language: &str) -> Result<Self, Error> {
        let child = start_divvun_checker(data_file_path)?;

        Ok(Self {
            child,
            path: data_file_path.to_owned(),
            language: language.to_owned(),
            terminated: false,
        })
    }

    fn kill_child(&mut self) {
        match self.child.kill() {
            Ok(_) => {
                // This blocks and may cause issues if the child doesn't properly die
                match self.child.wait() {
                    Ok(_) => debug!("Child killed"),
                    Err(e) => error!("Failed to kill child: {} while waiting", e),
                }
            }
            Err(e) => error!("Failed to kill child: {}", e),
        };
    }
}

fn start_divvun_checker(data_file_path: &str) -> Result<Child, Error> {
    let process = Command::new("divvun-checker")
        .arg("-a")
        .arg(data_file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(process)
}

impl Actor for GramcheckExecutor {
    type Context = Context<Self>;
}

impl Supervised for GramcheckExecutor {
    fn restarting(&mut self, _ctx: &mut Context<GramcheckExecutor>) {
        if !self.terminated {
            warn!("Actor for {} died, restarting", &self.language);

            // Killing previous child. Reusing the child would be more eco-friendly,
            // but there seems no reliable way to check the status of the child
            self.kill_child();
            self.child = match start_divvun_checker(&self.path) {
                Ok(child) => child,
                Err(e) => {
                    error!("Failed to spawn child for language `{}`!", &self.language);
                    panic!(e)
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GramcheckRequest {
    pub text: String,
}

impl Message for GramcheckRequest {
    type Result = Result<GramcheckResponse, ApiError>;
}

impl Handler<GramcheckRequest> for GramcheckExecutor {
    type Result = Result<GramcheckResponse, ApiError>;

    fn handle(&mut self, msg: GramcheckRequest, ctx: &mut Self::Context) -> Self::Result {
        let stdin = match self.child.stdin.as_mut() {
            Some(r) => r,
            _ => {
                ctx.stop();
                return Err(ApiError {
                    message: "Failed to open stdin".into(),
                });
            }
        };

        let stdout = match self.child.stdout.as_mut() {
            Some(r) => r,
            _ => {
                ctx.stop();
                return Err(ApiError {
                    message: "Failed to open stdout".into(),
                });
            }
        };

        let mut stdout = BufReader::new(stdout);

        let cleaned_msg = msg.text.split('\n').next().ok_or_else(|| ApiError {
            message: "Invalid input".into(),
        })?;

        let mut line = String::new();

        if let Err(err) = stdin
            .write_all(cleaned_msg.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdout.read_line(&mut line))
        {
            // If anything here fails, restart the runner
            ctx.stop();
            return Err(err.into());
        }

        serde_json::from_str(&line).map_err(|err| ApiError {
            message: format!("error: {:?}, line: '{}'", &err, &line),
        })
    }
}

pub struct Die;

impl Message for Die {
    type Result = ();
}

impl Handler<Die> for GramcheckExecutor {
    type Result = ();

    fn handle(&mut self, _: Die, ctx: &mut Self::Context) -> Self::Result {
        self.kill_child();

        debug!(
            "Death message received, stopping actor for language `{}`",
            &self.language
        );

        self.terminated = true;

        // The actor will restart because it's supervised, but if no references remain
        // to the actor it will be dropped
        ctx.stop();
    }
}

#[derive(Deserialize, Serialize)]
pub struct GramcheckPreferencesResponse {
    pub error_tags: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GramcheckErrResponse {
    pub error_text: String,
    pub start_index: u32,
    pub end_index: u32,
    pub error_code: String,
    pub description: String,
    pub suggestions: Vec<String>,
    pub title: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GramcheckResponse {
    pub text: String,
    pub errs: Vec<GramcheckErrResponse>,
}

pub struct AsyncGramchecker {
    pub gramcheckers: Arc<RwLock<HashMap<String, Addr<GramcheckExecutor>>>>,
}

impl LanguageSuggestions for AsyncGramchecker {
    type Request = GramcheckRequest;
    type Response = GramcheckResponse;

    fn suggestions(
        &self,
        message: Self::Request,
        language: &str,
    ) -> Box<dyn Future<Item = Self::Response, Error = ApiError>> {
        let gramcheckers = self.gramcheckers.read();

        let gramchecker = match gramcheckers.get(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: format!("No grammar checker available for language {}", &language),
                }));
            }
        };

        let language = language.to_owned();

        Box::new(
            gramchecker
                .send(message)
                .map_err(move |err| ApiError {
                    message: format!(
                        "Something failed in the message delivery process for language {}: {}",
                        &language, err
                    ),
                })
                .unhoist(),
        )
    }

    fn add(&self, language: &str, path: &str) -> Box<dyn Future<Item = (), Error = ApiError>> {
        info!("Adding Grammar Checker for {}", language);

        let mut gramcheckers = self.gramcheckers.write();

        let gramchecker_path = path.to_owned();
        let owned_language = language.to_owned();
        let gramchecker = actix::Supervisor::start_in_arbiter(&actix::Arbiter::new(), move |_| {
            GramcheckExecutor::new(&gramchecker_path, &owned_language)
                .expect(&format!("not found: {}", &gramchecker_path))
        });

        gramcheckers.insert(language.to_owned(), gramchecker);

        Box::new(ok(()))
    }

    fn remove(&self, language: &str) -> Box<dyn Future<Item = (), Error = ApiError>> {
        info!("Removing Grammar Checker for {}", language);

        let mut gramcheckers = self.gramcheckers.write();

        let gramchecker = match gramcheckers.remove(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: format!("No grammar checker available for language {}", &language),
                }));
            }
        };

        let cloned_gramcheckers = Arc::clone(&self.gramcheckers);
        let language = language.to_owned();

        Box::new(
            gramchecker
                .send(Die)
                .map_err(move |err| {
                    // Put the address back in since we failed to send the die message
                    let mut cloned_gramcheckers = cloned_gramcheckers.write();
                    cloned_gramcheckers.insert(language.clone(), gramchecker);

                    ApiError {
                        message: format!(
                            "Something failed in the message delivery process for language {}: {}",
                            &language, err
                        ),
                    }
                })
                .and_then(|_| ok(())),
        )
    }
}

pub fn list_preferences(data_file_path: &str) -> Result<BTreeMap<String, String>, Error> {
    let process = Command::new("divvun-checker")
        .arg("-a")
        .arg(data_file_path)
        .arg("-p")
        .output()
        .expect("failed to run divvun-checker");

    let regex = Regex::new(r"- \[.\] ([^\s+]+)\s+(.+)$").expect("valid regex");
    let toggle_separator = "==== Toggles: ====";

    let categories: BTreeMap<String, String> = String::from_utf8(process.stdout)
        .map_err(to_io_err)?
        .lines()
        .skip_while(|&l| l != toggle_separator)
        .skip(1)
        // temporary solution
        .skip_while(|&l| l != toggle_separator)
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

fn to_io_err(cause: impl ToString) -> Error {
    std::io::Error::new(std::io::ErrorKind::Other, cause.to_string())
}

#[cfg(test)]
mod test {
    use serde_json::json;

    #[test]
    fn test_foo() {
        let _some_data = json!({"errs":[["heno",0,4,"typo","Čállinmeattáhus",[]]],"text":"heno."});
    }
}
