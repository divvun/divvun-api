use std::sync::Arc;

use actix::prelude::*;
use divvunspell::archive::SpellerArchive;
use futures::future::{err, Future};
use hashbrown::HashMap;
use serde_derive::{Deserialize, Serialize};

use crate::server::state::{ApiError, SpellingSuggestions, UnhoistFutureExt};
use divvunspell::speller::SpellerConfig;

pub struct DivvunSpellExecutor(pub SpellerArchive);

impl Actor for DivvunSpellExecutor {
    type Context = Context<Self>;
}

impl actix::Supervised for DivvunSpellExecutor {
    fn restarting(&mut self, _ctx: &mut Context<DivvunSpellExecutor>) {
        println!("restarting");
    }
}

#[derive(Deserialize, Debug)]
pub struct SpellerRequest {
    pub word: String,
}

#[derive(Deserialize, Serialize, Debug)]
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
        let speller = self.0.speller();

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
        ctx.stop();
    }
}

pub struct AsyncSpeller {
    pub spellers: HashMap<String, Addr<DivvunSpellExecutor>>,
}

impl SpellingSuggestions for AsyncSpeller {
    fn spelling_suggestions(
        &self,
        message: SpellerRequest,
        language: &str,
    ) -> Box<Future<Item = SpellerResponse, Error = ApiError>> {
        let speller = match self.spellers.get(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: "No speller for that language".to_owned(),
                }));
            }
        };

        Box::new(
            speller
                .send(message)
                .map_err(|err| ApiError {
                    message: format!("Something failed in the message delivery process: {}", err),
                })
                .unhoist(),
        )
    }
}
