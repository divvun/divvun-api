use actix::prelude::*;
use actix_web::{
    http::header, http::Method, middleware, middleware::cors::Cors, server::HttpServer, App,
};
use sentry_actix::SentryMiddleware;

use hashbrown::HashMap;
use failure::Fail;
use serde_derive::Serialize;

use std::env;
use std::collections::BTreeMap;

use divvunspell::archive::SpellerArchive;

use crate::grammar::{get_gramcheck_preferences, list_preferences, post_gramcheck, GramcheckExecutor};
use crate::config::Config;
use crate::speller::{post_speller, DivvunSpellExecutor};
use crate::data_files::{get_available_languages, get_data_files, DataFileType};
use crate::query::graphiql;

#[derive(Fail, Debug, Serialize)]
#[fail(display = "api error")]
pub struct ApiError {
    pub message: String,
}

pub struct State {
    pub spellers: HashMap<String, Addr<DivvunSpellExecutor>>,
    pub gramcheckers: HashMap<String, Addr<GramcheckExecutor>>,
    pub gramcheck_preferences: HashMap<String, BTreeMap<String, String>>,
}

impl actix_web::error::ResponseError for ApiError {}

pub fn start_server(config: &Config) {
    let _guard = sentry::init(config.sentry_dsn.clone());
    env::set_var("RUST_BACKTRACE", "1");
    sentry::integrations::panic::register_panic_handler();

    let sys = actix::System::new("divvun-api");

    HttpServer::new(move || {
        App::with_state(get_state())
            .middleware(middleware::Logger::default())
            .middleware(SentryMiddleware::builder().emit_header(true).finish())
            .configure(|app| {
                Cors::for_app(app)
                    .send_wildcard()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
                    .resource("/graphiql", |r| {
                        r.method(Method::GET).f(graphiql);
                    })
                    .resource("/grammar/{languageCode}", |r| {
                        r.method(Method::POST).with_async(post_gramcheck);
                    })
                    .resource("/preferences/grammar/{languageCode}", |r| {
                        r.method(Method::GET).with_async(get_gramcheck_preferences);
                    })
                    .resource("/speller/{languageCode}", |r| {
                        r.method(Method::POST).with_async(post_speller);
                    })
                    .resource("/languages", |r| {
                        r.method(Method::GET).f(get_available_languages);
                    })
                    .register()
            })
    })
        .workers(4)
        .bind(&config.addr)
        .unwrap()
        .start();

    println!("Started http server: {}", &config.addr);
    let _ = sys.run();
}

fn get_state() -> State {
    let grammar_data_files = get_data_files(DataFileType::Grammar).unwrap_or_else(|e| {
        eprintln!("Error getting grammar data files: {}", e);
        vec![]
    });

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

    State {
        spellers,
        gramcheckers,
        gramcheck_preferences,
    }
}
