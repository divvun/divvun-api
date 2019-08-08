use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use actix::Addr;
use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use failure::Fail;
use futures::future::{err, ok, Future};
use hashbrown::HashMap;
use log::error;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::Config;
use crate::file_utils::get_file_info;
use crate::graphql::schema::create_schema;
use crate::graphql::schema::Schema;
use crate::language::data_files::{get_data_files, DataFileType};
use crate::language::grammar::{
    list_preferences, AsyncGramchecker, GramcheckExecutor, GramcheckOutput, GramcheckRequest,
};
use crate::language::hyphenation::{
    AsyncHyphenation, HyphenationExecutor, HyphenationRequest, HyphenationResponse,
};
use crate::language::speller::{
    AsyncSpeller, DivvunSpellExecutor, SpellerRequest, SpellerResponse,
};

#[derive(Fail, Debug, Deserialize, Serialize, Clone)]
#[fail(display = "api error")]
pub struct ApiError {
    pub message: String,
}

impl From<io::Error> for ApiError {
    fn from(item: io::Error) -> Self {
        ApiError {
            message: item.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for ApiError {
    fn from(item: std::string::FromUtf8Error) -> Self {
        ApiError {
            message: item.to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn render_response(&self) -> HttpResponse {
        error!("{}", self.message);
        return HttpResponse::InternalServerError()
            .content_type("application/json")
            .json(json!({ "message": self.message }));
    }
}

pub struct LanguageFunctions {
    pub spelling_suggestions: Box<dyn SpellingSuggestions>,
    pub grammar_suggestions: Box<dyn GrammarSuggestions>,
    pub hyphenation_suggestions: Box<dyn HyphenationSuggestions>,
}

pub trait SpellingSuggestions: Send + Sync {
    fn spelling_suggestions(
        &self,
        message: SpellerRequest,
        language: &str,
    ) -> Box<dyn Future<Item = SpellerResponse, Error = ApiError>>;
    fn add(&self, language: &str, path: &str) -> Box<dyn Future<Item = (), Error = ApiError>>;
    fn remove(&self, language: &str) -> Box<dyn Future<Item = (), Error = ApiError>>;
}

pub trait GrammarSuggestions: Send + Sync {
    fn grammar_suggestions(
        &self,
        message: GramcheckRequest,
        language: &str,
    ) -> Box<dyn Future<Item = GramcheckOutput, Error = ApiError>>;
    fn add(&self, language: &str, path: &str) -> Box<dyn Future<Item = (), Error = ApiError>>;
    fn remove(&self, language: &str) -> Box<dyn Future<Item = (), Error = ApiError>>;
}

pub trait HyphenationSuggestions: Send + Sync {
    fn hyphenation_suggestions(
        &self,
        message: HyphenationRequest,
        language: &str,
    ) -> Box<dyn Future<Item = HyphenationResponse, Error = ApiError>>;
    fn add(&self, language: &str, path: &str) -> Box<dyn Future<Item = (), Error = ApiError>>;
    fn remove(&self, language: &str) -> Box<dyn Future<Item = (), Error = ApiError>>;
}

pub trait UnhoistFutureExt<U, E> {
    fn unhoist(self) -> Box<dyn Future<Item = U, Error = E>>;
}

impl<T: 'static, U: 'static, E: 'static> UnhoistFutureExt<U, E> for T
where
    T: Future<Item = Result<U, E>, Error = E>,
{
    fn unhoist(self) -> Box<dyn Future<Item = U, Error = E>> {
        Box::new(self.and_then(|res| match res {
            Ok(result) => ok(result),
            Err(e) => err(e),
        }))
    }
}

pub type State = Arc<InnerState>;

pub struct InnerState {
    pub config: Config,
    pub graphql_schema: Schema,
    pub language_functions: LanguageFunctions,
    pub gramcheck_preferences: Arc<RwLock<HashMap<String, BTreeMap<String, String>>>>,
}

pub fn create_state(config: &Config) -> State {
    let grammar_data_files = get_data_files(config.data_file_dir.as_path(), DataFileType::Grammar)
        .unwrap_or_else(|e| {
            eprintln!("Error getting grammar data files: {}", e);
            vec![]
        });

    Arc::new(InnerState {
        config: config.clone(),
        graphql_schema: create_schema(),
        language_functions: LanguageFunctions {
            spelling_suggestions: Box::new(get_speller(config)),
            grammar_suggestions: Box::new(get_gramchecker(&grammar_data_files)),
            hyphenation_suggestions: Box::new(get_hyphenation(config)),
        },
        gramcheck_preferences: Arc::new(RwLock::new(get_gramcheck_preferences(
            &grammar_data_files,
        ))),
    })
}

fn get_speller(config: &Config) -> AsyncSpeller {
    let spelling_data_files =
        get_data_files(config.data_file_dir.as_path(), DataFileType::Spelling).unwrap_or_else(
            |e| {
                eprintln!("Error getting spelling data files: {}", e);
                vec![]
            },
        );

    let speller = AsyncSpeller {
        spellers: Arc::new(RwLock::new(
            HashMap::<String, Addr<DivvunSpellExecutor>>::new(),
        )),
    };

    for file in spelling_data_files {
        if let Some(file_info) = get_file_info(&file) {
            speller.add(file_info.stem, file_info.path);
        }
    }

    speller
}

fn get_gramchecker(grammar_data_files: &Vec<PathBuf>) -> AsyncGramchecker {
    let gramchecker = AsyncGramchecker {
        gramcheckers: Arc::new(RwLock::new(
            HashMap::<String, Addr<GramcheckExecutor>>::new(),
        )),
    };

    for file in grammar_data_files {
        if let Some(file_info) = get_file_info(&file) {
            gramchecker.add(file_info.stem, file_info.path);
        }
    }

    gramchecker
}

fn get_hyphenation(config: &Config) -> AsyncHyphenation {
    let hyphenation_data_files =
        get_data_files(config.data_file_dir.as_path(), DataFileType::Hyphenation).unwrap_or_else(
            |e| {
                eprintln!("Error getting hyphenation data files: {}", e);
                vec![]
            },
        );

    let hyphenator = AsyncHyphenation {
        hyphenators: Arc::new(RwLock::new(
            HashMap::<String, Addr<HyphenationExecutor>>::new(),
        )),
    };

    for file in hyphenation_data_files {
        if let Some(file_info) = get_file_info(&file) {
            hyphenator.add(file_info.stem, file_info.path);
        }
    }

    hyphenator
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
