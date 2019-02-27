use actix_web::{App, http::Method, server::HttpServer, middleware};
use actix::prelude::*;
use divvunspell::archive::{SpellerArchive};
use failure::Fail;
use hashbrown::HashMap;

use std::env;
use dotenv::dotenv;
use dotenv_codegen::{dotenv, expand_dotenv};

use sentry;
use sentry_actix::SentryMiddleware;

mod speller;
mod grammar;
mod data_files;

use speller::{DivvunSpellExecutor, post_speller};
use grammar::{GramcheckExecutor, post_gramcheck};
use data_files::{get_data_files, DataFileType};

#[derive(Fail, Debug)]
#[fail(display="api error")]
pub struct ApiError {
   pub message: &'static str
}

pub struct State {
    spellers: HashMap<String, Addr<DivvunSpellExecutor>>,
    gramcheckers: HashMap<String, Addr<GramcheckExecutor>>,
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
                let lang_code = f.file_stem().expect(&format!("oops, didn't find a file stem for {:?}", f)).to_str().unwrap();

                (lang_code.into(), SyncArbiter::start(3, move || {
                    let speller_path = f.to_str().unwrap();
                    let ar = SpellerArchive::new(speller_path);
                    DivvunSpellExecutor(ar.unwrap())
                }))
            })
            .collect();

        // Start 3 parallel grammar checker executors
        let gramcheckers = grammar_data_files
            .into_iter()
            .map(|f| {
                let lang_code = f.file_stem().unwrap().to_str().unwrap();
                
                (lang_code.into(), SyncArbiter::start(3, move || {
                let grammar_checker_path = f.to_str().unwrap();
                    GramcheckExecutor::new(grammar_checker_path).unwrap()
                }))
            })
            .collect();

        let state = State { spellers, gramcheckers };
        App::with_state(state)
            .middleware(middleware::Logger::default())
            .middleware(SentryMiddleware::builder().emit_header(true).finish())
            .resource("/speller/{languageCode}", |r| r.method(Method::POST).with_async(post_speller))
            .resource("/grammar/{languageCode}", |r| r.method(Method::POST).with_async(post_gramcheck))
    })
    .bind("127.0.0.1:8080").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
