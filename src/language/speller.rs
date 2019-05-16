use hashbrown::HashMap;
use actix::prelude::*;
use divvunspell::archive::SpellerArchive;
use futures::future::{err, Future};
use serde_derive::{Deserialize, Serialize};

use crate::server::state::{SpellingSuggestions, ApiError};

pub struct DivvunSpellExecutor(pub SpellerArchive);

impl Actor for DivvunSpellExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Deserialize, Debug)]
pub struct SpellerRequest {
    pub word: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpellerResponse {
    pub word: String,
    pub suggestions: Vec<String>,
}

impl Message for SpellerRequest {
    type Result = Result<SpellerResponse, ApiError>;
}

impl Handler<SpellerRequest> for DivvunSpellExecutor {
    type Result = Result<SpellerResponse, ApiError>;

    fn handle(&mut self, msg: SpellerRequest, _: &mut Self::Context) -> Self::Result {
        let suggestions = self
            .0
            .speller()
            .suggest(&msg.word)
            .into_iter()
            .map(|m| m.value)
            .collect();
        Ok(SpellerResponse {
            word: msg.word,
            suggestions,
        })
    }
}

pub struct AsyncSpeller {
    pub spellers: HashMap<String, Addr<DivvunSpellExecutor>>,
}

impl SpellingSuggestions for AsyncSpeller {
    fn spelling_suggestions(&self, message: SpellerRequest, language: &str)
        -> Box<Future<Item=Result<SpellerResponse, ApiError>, Error=ApiError>> {

        let speller = match self.spellers.get(language) {
            Some(s) => s,
            None => {
                return Box::new(err(ApiError {
                    message: "No speller for that language".to_owned()
                }));
            }
        };

        Box::new(
            speller
                .send(message)
                .map_err(|err|
                    ApiError { message: format!("Something failed in the message delivery process: {}", err) }
        ))
    }
}
