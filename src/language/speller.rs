use std::sync::Arc;

use actix::prelude::*;
use divvunspell::archive::SpellerArchive;
use futures::future::{err, ok, Future};
use hashbrown::HashMap;
use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::server::state::{LanguageSuggestions, UnhoistFutureExt};
use crate::error::ApiError;
use divvunspell::speller::suggestion::Suggestion;
use divvunspell::speller::SpellerConfig;
use divvunspell::tokenizer::Tokenize;

pub struct DivvunSpellExecutor {
    pub speller_archive: SpellerArchive,
    pub language: String,
    pub terminated: bool,
}

impl Actor for DivvunSpellExecutor {
    type Context = Context<Self>;
}

impl actix::Supervised for DivvunSpellExecutor {
    fn restarting(&mut self, _ctx: &mut Context<DivvunSpellExecutor>) {
        if !self.terminated {
            warn!("Actor for {} died, restarting", &self.language);
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SpellerRequest {
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpellerResponse {
    pub text: String,
    pub results: Vec<SpellerResult>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpellerResult {
    pub word: String,
    pub is_correct: bool,
    pub suggestions: Vec<Suggestion>,
}

impl Message for SpellerRequest {
    type Result = Result<SpellerResponse, ApiError>;
}

impl Handler<SpellerRequest> for DivvunSpellExecutor {
    type Result = Result<SpellerResponse, ApiError>;

    fn handle(&mut self, msg: SpellerRequest, _: &mut Self::Context) -> Self::Result {
        let speller = self.speller_archive.speller();

        let cloned_text = msg.text.clone();
        let words = cloned_text.words().into_iter();

        let results: Vec<SpellerResult> = words
            .map(|word| {
                let cloned_speller = self.speller_archive.speller().clone();
                let is_correct = Arc::clone(&speller).is_correct(word);

                let suggestions = cloned_speller
                    .suggest_with_config(
                        word,
                        &SpellerConfig {
                            n_best: Some(5),
                            max_weight: Some(10000f32),
                            beam: None,
                            with_caps: true,
                            pool_start: 128,
                            pool_max: 128,
                            seen_node_sample_rate: 20,
                        },
                    )
                    .into_iter()
                    .collect();

                SpellerResult {
                    word: word.to_owned(),
                    is_correct,
                    suggestions,
                }
            })
            .collect();

        Ok(SpellerResponse {
            text: cloned_text.clone(),
            results,
        })
    }
}

#[derive(Message)]
struct Die;

impl Handler<Die> for DivvunSpellExecutor {
    type Result = ();

    fn handle(&mut self, _: Die, ctx: &mut Context<DivvunSpellExecutor>) {
        // The actor will restart because it's supervised, but if no references remain
        // to the actor it will be dropped
        ctx.stop();
    }
}

pub struct AsyncSpeller {
    pub spellers: Arc<RwLock<HashMap<String, Addr<DivvunSpellExecutor>>>>,
}

impl LanguageSuggestions for AsyncSpeller {
    type Request = SpellerRequest;
    type Response = SpellerResponse;

    fn suggestions(
        &self,
        message: Self::Request,
        language: &str,
    ) -> Box<dyn Future<Item = Self::Response, Error = ApiError>> {
        let lock = self.spellers.read();

        let speller = match lock.get(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: format!("No speller available for language {}", &language),
                }));
            }
        };

        let language = language.to_owned();

        Box::new(
            speller
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
        info!("Adding Speller for {}", language);

        let mut lock = self.spellers.write();

        let speller_path = path.to_owned();
        let ar = SpellerArchive::new(&speller_path);

        let owned_language = language.to_owned();
        let speller = actix::Supervisor::start_in_arbiter(&actix::Arbiter::new(), move |_| {
            DivvunSpellExecutor {
                speller_archive: ar.unwrap(),
                language: owned_language,
                terminated: false,
            }
        });

        lock.insert(language.to_owned(), speller);

        Box::new(ok(()))
    }

    fn remove(&self, language: &str) -> Box<dyn Future<Item = (), Error = ApiError>> {
        info!("Removing Speller for {}", language);

        let mut lock = self.spellers.write();

        let speller = match lock.remove(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: format!("No speller available for language {}", &language),
                }));
            }
        };

        let cloned_spellers = Arc::clone(&self.spellers);
        let language = language.to_owned();

        Box::new(
            speller
                .send(Die)
                .map_err(move |err| {
                    // Put the address back in since we failed to send the die message
                    let mut lock = cloned_spellers.write();
                    lock.insert(language.clone(), speller);

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
