use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use divvunspell::archive::SpellerArchive;
use failure::Fail;
use futures::future::{err, ok, Future};
use hashbrown::HashMap;
use serde_derive::Serialize;

use crate::graphql::schema::create_schema;
use crate::graphql::schema::Schema;
use crate::language::data_files::{get_data_files, DataFileType};
use crate::language::grammar::{
    list_preferences, AsyncGramchecker, GramcheckExecutor, GramcheckOutput, GramcheckRequest,
};
use crate::language::speller::{
    AsyncSpeller, DivvunSpellExecutor, SpellerRequest, SpellerResponse,
};
use serde_json::json;
use std::path::PathBuf;

#[derive(Fail, Debug, Serialize)]
#[fail(display = "api error")]
pub struct ApiError {
    pub message: String,
}

impl ResponseError for ApiError {
    fn render_response(&self) -> HttpResponse {
        return HttpResponse::InternalServerError()
            .content_type("application/json")
            .json(json!({ "message": self.message }));
    }
}

pub struct LanguageFunctions {
    pub spelling_suggestions: Box<SpellingSuggestions>,
    pub grammar_suggestions: Box<GrammarSuggestions>,
}

pub trait SpellingSuggestions: Send + Sync {
    fn spelling_suggestions(
        &self,
        message: SpellerRequest,
        language: &str,
    ) -> Box<Future<Item = SpellerResponse, Error = ApiError>>;
}

pub trait GrammarSuggestions: Send + Sync {
    fn grammar_suggestions(
        &self,
        message: GramcheckRequest,
        language: &str,
    ) -> Box<Future<Item = GramcheckOutput, Error = ApiError>>;
    fn add(&self, language: &str, path: PathBuf) -> Box<Future<Item = String, Error = ApiError>>;
    fn remove(&self, language: &str) -> Box<Future<Item = String, Error = ApiError>>;
}

pub trait UnhoistFutureExt<U, E> {
    fn unhoist(self) -> Box<Future<Item = U, Error = E>>;
}

impl<T: 'static, U: 'static, E: 'static> UnhoistFutureExt<U, E> for T
where
    T: Future<Item = Result<U, E>, Error = E>,
{
    fn unhoist(self) -> Box<Future<Item = U, Error = E>> {
        Box::new(self.and_then(|res| match res {
            Ok(result) => ok(result),
            Err(e) => err(e),
        }))
    }
}

pub type State = Arc<InnerState>;

pub struct InnerState {
    pub graphql_schema: Schema,
    pub language_functions: LanguageFunctions,
    pub gramcheck_preferences: HashMap<String, BTreeMap<String, String>>,
}

pub fn create_state() -> State {
    let grammar_data_files = get_data_files(DataFileType::Grammar).unwrap_or_else(|e| {
        eprintln!("Error getting grammar data files: {}", e);
        vec![]
    });

    Arc::new(InnerState {
        graphql_schema: create_schema(),
        language_functions: LanguageFunctions {
            spelling_suggestions: Box::new(get_speller()),
            grammar_suggestions: Box::new(get_gramchecker(&grammar_data_files)),
        },
        gramcheck_preferences: get_gramcheck_preferences(&grammar_data_files),
    })
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

            let speller_path = f.to_str().unwrap();
            let ar = SpellerArchive::new(speller_path);

            (
                lang_code.into(),
                actix::Supervisor::start_in_arbiter(&actix::Arbiter::new(), |_| {
                    DivvunSpellExecutor(ar.unwrap())
                }),
            )
        })
        .collect();

    AsyncSpeller { spellers }
}

fn get_gramchecker(grammar_data_files: &Vec<PathBuf>) -> AsyncGramchecker {
    let gramcheckers = grammar_data_files
        .to_owned()
        .into_iter()
        .map(|f| {
            let lang_code = f.file_stem().unwrap().to_str().unwrap();

            let grammar_checker_path = f.to_str().unwrap().to_owned();

            (
                lang_code.into(),
                actix::Supervisor::start_in_arbiter(&actix::Arbiter::new(), move |_| {
                    GramcheckExecutor::new(&grammar_checker_path)
                        .expect(&format!("not found: {}", grammar_checker_path))
                }),
            )
        })
        .collect();

    AsyncGramchecker {
        gramcheckers: Arc::new(RwLock::new(gramcheckers)),
    }
}

fn get_gramcheck_preferences(
    grammar_data_files: &Vec<PathBuf>,
) -> HashMap<String, BTreeMap<String, String>> {
    let gramcheck_preferences = grammar_data_files
        .into_iter()
        .map(|f| {
            let grammar_checker_path = f.to_str().unwrap();
            let lang_code = f.file_stem().unwrap().to_str().unwrap();
            (
                lang_code.into(),
                list_preferences(grammar_checker_path).unwrap(),
            )
        })
        .collect();

    gramcheck_preferences
}
