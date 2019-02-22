use actix_web::{server, App, HttpRequest, HttpResponse, http::Method, server::HttpServer, AsyncResponder, middleware};
use actix::prelude::*;
use divvunspell::archive::{SpellerArchive, SpellerArchiveError};
use failure::Fail;
use hashbrown::HashMap;

mod speller;
mod grammar;

use speller::{DivvunSpellExecutor, post_speller};
use grammar::{GramcheckExecutor, post_gramcheck};

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
    let sys = actix::System::new("divvun-api");

    // Start http server
    HttpServer::new(move || {

        // Start 3 parallel speller executors
        let spellers = {
            let mut s = HashMap::new();
            s.insert("se".into(), SyncArbiter::start(3, || {
                let ar = SpellerArchive::new("./se-stored.zhfst");
                DivvunSpellExecutor(ar.unwrap())
            }));
            s
        };

        let gramcheckers = {
            let mut s = HashMap::new();
            s.insert("se".into(), SyncArbiter::start(3, || {
                GramcheckExecutor::new("se").unwrap()
            }));
            s
        };

        let state = State { spellers, gramcheckers };
        App::with_state(state)
            .middleware(middleware::Logger::default())
            .resource("/speller/{languageCode}", |r| r.method(Method::POST).with_async(post_speller))
            .resource("/grammar/{languageCode}", |r| r.method(Method::POST).with_async(post_gramcheck))
    })
    .bind("127.0.0.1:8080").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}