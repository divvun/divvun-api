use std::env;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{http::header, middleware, web, App, HttpServer};

pub mod state;

use self::state::State;
use crate::config::Config;
use crate::graphql::handlers::{graphiql, graphql};

use crate::language::handlers::{
    get_available_languages_handler, get_gramcheck_preferences_handler, gramchecker_handler,
    speller_handler,
};

pub fn start_server(state: State, config: &Config) -> Server {
    env::set_var("RUST_BACKTRACE", "1");

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .send_wildcard()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            .service(web::resource("/graphql").route(web::post().to_async(graphql)))
            .service(
                web::resource("/speller/{languageCode}")
                    .route(web::post().to_async(speller_handler)),
            )
            .service(
                web::resource("/grammar/{languageCode}")
                    .route(web::post().to_async(gramchecker_handler)),
            )
            .service(
                web::resource("/preferences/grammar/{languageCode}")
                    .route(web::get().to_async(get_gramcheck_preferences_handler)),
            )
            .service(
                web::resource("/languages").route(web::get().to(get_available_languages_handler)),
            )
    })
    .workers(4)
    .bind(&config.addr)
    .unwrap()
    .start()
}
