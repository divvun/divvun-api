use gotham::router::Router;
use gotham::router::builder::*;
use gotham::handler::IntoResponse;
use gotham::state::State;
use gotham_derive::StateData;
use gotham_derive::StaticResponseExtender;
use serde::{Deserialize, Serialize};
use hyper::{Response, Body, StatusCode};
use gotham::helpers::http::response::create_response;


#[derive(Serialize)]
struct GrammarCheckResult {
    text: String,
    errs: Vec<GrammarCheckError>,
}

#[derive(Serialize)]
struct GrammarCheckError {
    error_text: String,
    start_index: u32,
    end_index: u32,
    error_code: String,
    description: String,
    suggestions: Vec<String>,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct GrammarCheckParams {
    lang: String,
}

#[derive(Serialize)]
struct GrammarCheckResponse {
    results: Vec<GrammarCheckResult>,
}

impl IntoResponse for GrammarCheckResponse {
    fn into_response(self, state: &State) -> Response<Body> {
        create_response(
            state,
            StatusCode::OK,
            mime::APPLICATION_JSON,
            serde_json::to_string(&self).expect("serialized"),
        )
    }
}

fn grammar_check_handler(state: State) -> (State, GrammarCheckResponse) {
    let response = GrammarCheckResponse {
        results: Vec::new(),
    };

    (state, response)
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct SpellingCheckParams {
    lang: String,
}

#[derive(Serialize)]
struct SpellingCheckResponse {

}

impl IntoResponse for SpellingCheckResponse {
    fn into_response(self, state: &State) -> Response<Body> {
        create_response(
            state,
            StatusCode::OK,
            mime::APPLICATION_JSON,
            serde_json::to_string(&self).expect("serialized"),
        )
    }
}

fn spelling_check_handler(state: State) -> (State, SpellingCheckResponse) {
    let response = SpellingCheckResponse {

    };

    (state, response)
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .post("/grammar/:lang")
            .with_path_extractor::<GrammarCheckParams>()
            .to(grammar_check_handler);
        
        route
            .post("/spelling/:lang")
            .with_path_extractor::<SpellingCheckParams>()
            .to(spelling_check_handler);
    })
}

fn main() {
    let addr = "127.0.0.1:8666";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}
