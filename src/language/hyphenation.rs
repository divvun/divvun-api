use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

use actix::prelude::*;
use futures::future::{err, ok, Future};
use hashbrown::HashMap;
use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use divvunspell::tokenizer::Tokenize;

use crate::server::state::{LanguageSuggestions, UnhoistFutureExt};
use crate::error::ApiError;

pub struct HyphenationExecutor {
    pub path: String,
    pub language: String,
    pub terminated: bool,
}

impl Actor for HyphenationExecutor {
    type Context = Context<Self>;
}

impl actix::Supervised for HyphenationExecutor {
    fn restarting(&mut self, _ctx: &mut Context<HyphenationExecutor>) {
        if !self.terminated {
            warn!("Hyphenation actor for {} died, restarting", &self.language);
        }
    }
}

#[derive(Message)]
struct Die;

impl Handler<Die> for HyphenationExecutor {
    type Result = ();

    fn handle(&mut self, _: Die, ctx: &mut Context<HyphenationExecutor>) {
        // The actor will restart because it's supervised, but if no references remain
        // to the actor it will be dropped
        ctx.stop();
    }
}

#[derive(Debug, Deserialize)]
pub struct HyphenationRequest {
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HyphenationResponse {
    pub text: String,
    pub results: Vec<HyphenationResult>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HyphenationResult {
    pub word: String,
    pub patterns: Vec<HyphenationPattern>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HyphenationPattern {
    pub value: String,
    pub weight: String,
}

impl Message for HyphenationRequest {
    type Result = Result<HyphenationResponse, ApiError>;
}

impl Handler<HyphenationRequest> for HyphenationExecutor {
    type Result = Result<HyphenationResponse, ApiError>;

    fn handle(&mut self, msg: HyphenationRequest, _: &mut Self::Context) -> Self::Result {
        let cloned_text = msg.text.clone();
        let words = cloned_text.words().into_iter();

        let results = words
            .map(|word| {
                let mut hfst_child = Command::new("hfst-lookup")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .arg("-n")
                    .arg("1")
                    .arg("-q")
                    .arg(&self.path)
                    .spawn()?;

                {
                    let hfst_in = hfst_child.stdin.as_mut().unwrap();
                    hfst_in.write_all(&word.as_bytes())?;
                }

                let result = String::from_utf8(hfst_child.wait_with_output()?.stdout)?
                    .trim()
                    .to_string();

                let suggestions = result
                    .lines()
                    .map(|line| {
                        let components: Vec<&str> = line.split("\t").collect();
                        if components.len() < 3 {
                            panic!("hfst-lookup returning unexpected number of tokens per word");
                        }

                        HyphenationPattern {
                            value: components[1].to_owned(),
                            weight: components[2].to_owned(),
                        }
                    })
                    .collect();

                Ok(HyphenationResult {
                    word: word.to_owned(),
                    patterns: suggestions,
                })
            })
            .collect::<Result<Vec<HyphenationResult>, ApiError>>()?;

        Ok(HyphenationResponse {
            text: cloned_text,
            results,
        })
    }
}

pub struct AsyncHyphenator {
    pub hyphenators: Arc<RwLock<HashMap<String, Addr<HyphenationExecutor>>>>,
}

impl LanguageSuggestions for AsyncHyphenator {
    type Request = HyphenationRequest;
    type Response = HyphenationResponse;

    fn suggestions(
        &self,
        message: Self::Request,
        language: &str,
    ) -> Box<dyn Future<Item = Self::Response, Error = ApiError>> {
        let lock = self.hyphenators.read();

        let hyphenator = match lock.get(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: format!("No hyphenator available for language {}", &language),
                }));
            }
        };

        let language = language.to_owned();

        Box::new(
            hyphenator
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
        info!("Adding Hyphenator for {}", language);

        let mut lock = self.hyphenators.write();

        let hyphenator_path = path.to_owned();

        let owned_language = language.to_owned();
        let hyphenator = actix::Supervisor::start_in_arbiter(&actix::Arbiter::new(), move |_| {
            HyphenationExecutor {
                path: hyphenator_path,
                language: owned_language,
                terminated: false,
            }
        });

        lock.insert(language.to_owned(), hyphenator);

        Box::new(ok(()))
    }

    fn remove(&self, language: &str) -> Box<dyn Future<Item = (), Error = ApiError>> {
        info!("Removing Hyphenator for {}", language);

        let mut lock = self.hyphenators.write();

        let hyphenator = match lock.remove(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: format!("No hyphenator available for language {}", &language),
                }));
            }
        };

        let cloned_hyphenators = Arc::clone(&self.hyphenators);
        let language = language.to_owned();

        Box::new(
            hyphenator
                .send(Die)
                .map_err(move |err| {
                    // Put the address back in since we failed to send the die message
                    let mut lock = cloned_hyphenators.write();
                    lock.insert(language.clone(), hyphenator);

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
