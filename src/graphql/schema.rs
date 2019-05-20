use juniper::{RootNode, EmptyMutation, FieldResult, graphql_object, GraphQLObject};
use futures::future::Future;

use crate::server::state::State;
use crate::language::grammar;
use crate::language::grammar::GramcheckRequest;
use crate::language::speller::SpellerRequest;

impl juniper::Context for State {}

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
    pub suggestions: Vec<String>,
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

graphql_object!(QueryRoot: State |&self| {
    field suggestions(&executor, text: String, language: String) -> FieldResult<Suggestions> {
        Ok(Suggestions { text, language })
    }
});

graphql_object!(Suggestions: State |&self| {
    description: "Text suggestions"

    field grammar(&executor) -> FieldResult<Grammar> {
        get_grammar_suggestions(executor.context(), &self.text, &self.language)
    }

    field speller(&executor) -> FieldResult<Speller> {
        get_speller_suggestions(executor.context(), &self.text, &self.language)
    }
});

fn get_grammar_suggestions(state: &State, text: &str, language: &str) -> FieldResult<Grammar> {
    let grammar_suggestions = state.language_functions.grammar_suggestions
        .grammar_suggestions(GramcheckRequest { text: text.to_owned() }, language)
        .wait();

    match grammar_suggestions {
        Ok(gram_output) => Ok(Grammar { errs: gram_output.errs
            .into_iter()
            .map(|response| {
                GramcheckErrResponse::from(response)
            }).collect() }),
        Err(error) => Err(error)?,
    }
}

fn get_speller_suggestions(state: &State, text: &str, language: &str) -> FieldResult<Speller> {
    let speller_suggestions = state.language_functions.spelling_suggestions
        .spelling_suggestions(SpellerRequest { word: text.to_owned() }, language)
        .wait();

    match speller_suggestions {
        Ok(speller_output) => Ok(Speller { suggestions: speller_output.suggestions }),
        Err(error) => Err(error)?
    }
}

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation<State>>;

pub fn create_schema() -> Schema {
     Schema::new(QueryRoot {}, EmptyMutation::new())
}
