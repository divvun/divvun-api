use actix::prelude::*;
use actix_web::{
    http::header, http::Method, middleware, middleware::cors::Cors, server::HttpServer, App,
};
use divvunspell::archive::SpellerArchive;
use failure::Fail;
use hashbrown::HashMap;
use serde_derive::Serialize;

use dotenv::dotenv;
use dotenv_codegen::{dotenv, expand_dotenv};
use std::env;

use sentry;
use sentry_actix::SentryMiddleware;

mod data_files;
mod grammar;
mod speller;

use data_files::{get_data_files, DataFileType};
use grammar::{get_gramcheck_preferences, list_preferences, post_gramcheck, GramcheckExecutor};
use speller::{post_speller, DivvunSpellExecutor};
use std::collections::BTreeMap;

#[derive(Fail, Debug, Serialize)]
#[fail(display = "api error")]
pub struct ApiError {
    pub message: String,
}

pub struct State {
    spellers: HashMap<String, Addr<DivvunSpellExecutor>>,
    gramcheckers: HashMap<String, Addr<GramcheckExecutor>>,
    gramcheck_preferences: HashMap<String, BTreeMap<String, String>>,
}

// Use default implementation for `error_response()` method
impl actix_web::error::ResponseError for ApiError {}

fn main() {
    dotenv().ok();
    let sentry_dsn = dotenv!("SENTRY_DSN");
    let _guard = sentry::init(sentry_dsn);
    env::set_var("RUST_BACKTRACE", "1");
    sentry::integrations::panic::register_panic_handler();

    let sys = actix::System::new("divvun-api");

    // Start http server
    HttpServer::new(move || {
        let grammar_data_files = get_data_files(DataFileType::Grammar).unwrap_or_else(|e| {
            eprintln!("Error getting grammar data files: {}", e);
            vec![]
        });

        let spelling_data_files = get_data_files(DataFileType::Spelling).unwrap_or_else(|e| {
            eprintln!("Error getting spelling data files: {}", e);
            vec![]
        });

        // Start 3 parallel speller executors
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

        // Start 3 parallel grammar checker executors
        let gramcheckers = grammar_data_files
            .to_owned()
            .into_iter()
            .map(|f| {
                let lang_code = f.file_stem().unwrap().to_str().unwrap();

                (
                    lang_code.into(),
                    SyncArbiter::start(1, move || {
                        let grammar_checker_path = f.to_str().unwrap();
                        GramcheckExecutor::new(grammar_checker_path).unwrap()
                    }),
                )
            })
            .collect();

        // Load available preferences for each language code
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

        let state = State {
            spellers,
            gramcheckers,
            gramcheck_preferences,
        };

        App::with_state(state)
            .middleware(middleware::Logger::default())
            .middleware(SentryMiddleware::builder().emit_header(true).finish())
            .configure(|app| {
                Cors::for_app(app)
                    .send_wildcard()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
                    .resource("/grammar/{languageCode}", |r| {
                        r.method(Method::POST).with_async(post_gramcheck);
                    })
                    .resource("/preferences/grammar/{languageCode}", |r| {
                        r.method(Method::GET).with_async(get_gramcheck_preferences);
                    })
                    .resource("/speller/{languageCode}", |r| {
                        r.method(Method::POST).with_async(post_speller);
                    })
                    .register()
            })
    })
    .workers(4)
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
