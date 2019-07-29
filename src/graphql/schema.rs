use futures::future::Future;
use juniper::{graphql_object, EmptyMutation, FieldResult, GraphQLObject, RootNode};

use crate::language::grammar::{self, GramcheckRequest};
use crate::language::speller::{self, SpellerRequest};
use crate::server::state::InnerState;
use divvunspell::speller::suggestion::Suggestion;

impl juniper::Context for InnerState {}

#[derive(Debug)]
pub struct Suggestions {
    text: String,
    language: String,
}

#[derive(GraphQLObject)]
pub struct Grammar {
    pub errs: Vec<GramcheckErrResponse>,
}

#[derive(GraphQLObject)]
pub struct Speller {
    pub results: Vec<SpellerResult>,
}

#[derive(GraphQLObject)]
pub struct SpellerResult {
    pub word: String,
    pub is_correct: bool,
    pub suggestions: Vec<SpellerSuggestion>,
}

impl From<speller::SpellerResult> for SpellerResult {
    fn from(item: speller::SpellerResult) -> Self {
        SpellerResult {
            word: item.word,
            is_correct: item.is_correct,
            suggestions: item.suggestions.into_iter().map(|suggestion| SpellerSuggestion::from(suggestion)).collect(),
        }
    }
}

#[derive(GraphQLObject)]
pub struct SpellerSuggestion {
    pub value: String,
    pub weight: f64,
}

impl From<Suggestion> for SpellerSuggestion {
    fn from(item: Suggestion) -> Self {
        SpellerSuggestion {
            value: item.value,
            weight: item.weight as f64,
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "Grammar Checker errors")]
pub struct GramcheckErrResponse {
    error_text: String,
    start_index: i32,
    end_index: i32,
    error_code: String,
    description: String,
    suggestions: Vec<String>,
    title: String,
}

impl From<grammar::GramcheckErrResponse> for GramcheckErrResponse {
    fn from(item: grammar::GramcheckErrResponse) -> Self {
        GramcheckErrResponse {
            error_text: item.error_text,
            start_index: item.start_index as i32,
            end_index: item.end_index as i32,
            error_code: item.error_code,
            description: item.description,
            suggestions: item.suggestions,
            title: item.title,
        }
    }
}

pub struct QueryRoot;

graphql_object!(QueryRoot: InnerState |&self| {
    field suggestions(&executor, text: String, language: String) -> FieldResult<Suggestions> {
        Ok(Suggestions { text, language })
    }
});

graphql_object!(Suggestions: InnerState |&self| {
    description: "Text suggestions"

    field grammar(&executor) -> FieldResult<Grammar> {
        get_grammar_suggestions(executor.context(), &self.text, &self.language)
    }

    field speller(&executor) -> FieldResult<Speller> {
        get_speller_suggestions(executor.context(), &self.text, &self.language)
    }
});

fn get_grammar_suggestions(state: &InnerState, text: &str, language: &str) -> FieldResult<Grammar> {
    let grammar_suggestions = state
        .language_functions
        .grammar_suggestions
        .grammar_suggestions(
            GramcheckRequest {
                text: text.to_owned(),
            },
            language,
        )
        .wait();

    match grammar_suggestions {
        Ok(gram_output) => Ok(Grammar {
            errs: gram_output
                .errs
                .into_iter()
                .map(|response| GramcheckErrResponse::from(response))
                .collect(),
        }),
        Err(error) => Err(error)?,
    }
}

fn get_speller_suggestions(state: &InnerState, text: &str, language: &str) -> FieldResult<Speller> {
    let speller_suggestions = state
        .language_functions
        .spelling_suggestions
        .spelling_suggestions(
            SpellerRequest {
                text: text.to_owned(),
            },
            language,
        )
        .wait();

    match speller_suggestions {
        Ok(speller_output) => Ok(Speller {
            results: speller_output.results.into_iter()
                .map(|suggestion| SpellerResult::from(suggestion)).collect(),
        }),
        Err(error) => Err(error)?,
    }
}

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation<InnerState>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, EmptyMutation::new())
}
