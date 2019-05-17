use juniper::{RootNode, EmptyMutation, FieldResult, graphql_object, GraphQLObject};
use serde_derive::{Deserialize, Serialize};

use crate::server::state::State;
use crate::language::grammar;

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

        Ok(Suggestions {
            grammar: vec![GramcheckErrResponse {
                error_text: "drt".to_owned(),
                start_index: 3,
                end_index: 12,
                error_code: "drt".to_owned(),
                description: "drt".to_owned(),
                suggestions: vec!["drt".to_owned()],
                title: "drt".to_owned(),
            }],
        })
    }
});

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation<State>>;

pub fn create_schema() -> Schema {
     Schema::new(QueryRoot {}, EmptyMutation::new())
}
