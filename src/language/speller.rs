use std::sync::Arc;

use actix::prelude::*;
use divvunspell::archive::SpellerArchive;
use futures::future::{err, ok, Future};
use hashbrown::HashMap;
use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::server::state::{ApiError, SpellingSuggestions, UnhoistFutureExt};
use divvunspell::speller::SpellerConfig;

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
    pub word: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpellerResponse {
    pub word: String,
    pub is_correct: bool,
    pub suggestions: Vec<String>,
}

impl Message for SpellerRequest {
    type Result = Result<SpellerResponse, ApiError>;
}

impl Handler<SpellerRequest> for DivvunSpellExecutor {
    type Result = Result<SpellerResponse, ApiError>;

    fn handle(&mut self, msg: SpellerRequest, _: &mut Self::Context) -> Self::Result {
        let speller = self.speller_archive.speller();

        let is_correct = Arc::clone(&speller).is_correct(&msg.word);

        let suggestions = speller
            .suggest_with_config(
                &msg.word,
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
            .map(|m| m.value)
            .collect();

        Ok(SpellerResponse {
            word: msg.word,
            is_correct,
            suggestions,
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

impl SpellingSuggestions for AsyncSpeller {
    fn spelling_suggestions(
        &self,
        message: SpellerRequest,
        language: &str,
    ) -> Box<dyn Future<Item = SpellerResponse, Error = ApiError>> {
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
