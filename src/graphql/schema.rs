use juniper::{RootNode, EmptyMutation, FieldResult, graphql_object, GraphQLObject};
use serde_derive::{Deserialize, Serialize};
use futures::future::Future;

use crate::server::state::State;
use crate::language::grammar;
use crate::language::grammar::GramcheckRequest;

impl juniper::Context for State {}

#[derive(GraphQLObject)]
#[graphql(description = "Text suggestions")]
pub struct Suggestions {
    pub grammar: Vec<GramcheckErrResponse>,
}

#[derive(Deserialize, Serialize, GraphQLObject)]
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

graphql_object!(QueryRoot: State |&self| {
    field suggestions(&executor, text: String, language: String) -> FieldResult<Suggestions> {
        get_suggestions(executor.context(), &text, &language)
    }
});

fn get_suggestions(state: &State, text: &str, language: &str) -> FieldResult<Suggestions> {

    let grammar_suggestions = state.language_functions.grammar_suggestions
        .grammar_suggestions(GramcheckRequest { text: text.to_owned() }, language)
        .wait();

    match grammar_suggestions {
        Ok(gram_output) => Ok(Suggestions {
            grammar: gram_output.errs
                .into_iter()
                .map(|response| {
                    GramcheckErrResponse::from(response)
                }).collect()
        }),
        Err(error) => Err(error)?
    }
}

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation<State>>;

pub fn create_schema() -> Schema {
     Schema::new(QueryRoot {}, EmptyMutation::new())
}
