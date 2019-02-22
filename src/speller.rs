use actix_web::{server, App, HttpRequest, HttpResponse, Json, Path, State, http::Method, server::HttpServer, AsyncResponder};
use futures::future::{Future, result};
use actix::prelude::*;
use divvunspell::archive::{SpellerArchive, SpellerArchiveError};
use serde_derive::{Deserialize, Serialize};

use crate::{ApiError, State as AppState};

pub struct DivvunSpellExecutor(pub SpellerArchive);

impl Actor for DivvunSpellExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Deserialize, Debug)]
pub struct SpellerRequest {
    pub word: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpellerResponse {
    pub word: String,
    pub suggestions: Vec<String>
}

impl Message for SpellerRequest {
    type Result = Result<SpellerResponse, ApiError>;
}

impl Handler<SpellerRequest> for DivvunSpellExecutor {
    type Result = Result<SpellerResponse, ApiError>;

    fn handle(&mut self, msg: SpellerRequest, _: &mut Self::Context) -> Self::Result {
        let suggestions = self.0.speller().suggest(&msg.word).into_iter().map(|m| m.value).collect();
        Ok(SpellerResponse {
            word: msg.word,
            suggestions
        })
    }
}

/// Async handler
pub fn post_speller(body: Json<SpellerRequest>, language: Path<String>, state: State<AppState>) -> Box<Future<Item=HttpResponse, Error=actix_web::Error>> {
    let speller = match state.spellers.get(&*language) {
        Some(s) => s,
        None => {
            return result(Ok(HttpResponse::InternalServerError().into())).responder();
        }
    };

    speller.send(body.0)
        .from_err()
        .and_then(|res| {
            match res {
                Ok(result) => Ok(HttpResponse::Ok().json(result)),
                Err(_) => Ok(HttpResponse::InternalServerError().into())
            }
        })
        .responder()
}
