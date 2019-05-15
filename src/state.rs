use std::sync::Arc;

use failure::Fail;
use serde_derive::Serialize;
use futures::future::Future;
use actix::prelude::*;
use actix_web::error::ResponseError;
use divvunspell::archive::SpellerArchive;

use crate::graphql::schema::Schema;
use crate::speller::{AsyncSpeller, SpellerResponse, SpellerRequest, DivvunSpellExecutor};
use crate::grammar::{AsyncGramchecker, GramcheckOutput, GramcheckRequest, GramcheckExecutor};
use crate::graphql::schema::create_schema;
use crate::data_files::{get_available_languages, get_data_files, DataFileType};

#[derive(Fail, Debug, Serialize)]
#[fail(display = "api error")]
pub struct ApiError {
    pub message: String,
}

impl ResponseError for ApiError {}

pub struct LanguageFunctions {
    pub spelling_suggestions: Box<SpellingSuggestions>,
    pub grammar_suggesgions: Box<GrammarSuggestions>,
}

pub trait SpellingSuggestions {
    fn spelling_suggestions(&self, message: SpellerRequest, language: &str)
        -> Box<Future<Item = Result<SpellerResponse, ApiError>, Error = ApiError>>;
}

pub trait GrammarSuggestions {
    fn grammar_suggestions(&self, message: GramcheckRequest, language: &str)
        -> Box<Future<Item=Result<GramcheckOutput, ApiError>, Error=ApiError>>;
}

pub struct State {
    pub graphql_schema: Arc<Schema>,
    pub language_functions: LanguageFunctions,
}

pub fn create_state() -> State {
    State {
        graphql_schema: Arc::new(create_schema()),
        language_functions: LanguageFunctions {
            spelling_suggestions: Box::new(get_speller()),
            grammar_suggesgions: Box::new(get_gramchecker()),
        }
    }
}

fn get_speller() -> AsyncSpeller {
    let spelling_data_files = get_data_files(DataFileType::Spelling).unwrap_or_else(|e| {
        eprintln!("Error getting spelling data files: {}", e);
        vec![]
    });

    let spellers = spelling_data_files
        .into_iter()
        .map(|f| {
            let lang_code = f
                .file_stem()
                .expect(&format!("oops, didn't find a file stem for {:?}", f))
                .to_str()
                .unwrap();

            (
                lang_code.into(),
                SyncArbiter::start(1, move || {
                    let speller_path = f.to_str().unwrap();
                    let ar = SpellerArchive::new(speller_path);
                    DivvunSpellExecutor(ar.unwrap())
                }),
            )
        })
        .collect();

    AsyncSpeller { spellers }
}

fn get_gramchecker() -> AsyncGramchecker {
    let grammar_data_files = get_data_files(DataFileType::Grammar).unwrap_or_else(|e| {
        eprintln!("Error getting grammar data files: {}", e);
        vec![]
    });

    let gramcheckers = grammar_data_files
        .to_owned()
        .into_iter()
        .map(|f| {
            let lang_code = f.file_stem().unwrap().to_str().unwrap();

            (
                lang_code.into(),
                SyncArbiter::start(1, move || {
                    let grammar_checker_path = f.to_str().unwrap();
                    GramcheckExecutor::new(grammar_checker_path)
                        .expect(&format!("not found: {}", grammar_checker_path))
                }),
            )
        })
        .collect();

    AsyncGramchecker { gramcheckers }
}
