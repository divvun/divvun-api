use actix_web::{web, HttpResponse};

use futures::future::{result, Future};

use crate::server::state::State;

use super::data_files::{
    available_languages, AvailableLanguagesByType, AvailableLanguagesResponse, DataFileType,
};
use super::grammar::{GramcheckPreferencesResponse, GramcheckRequest};
use super::hyphenation::HyphenationRequest;
use super::speller::SpellerRequest;

pub fn get_available_languages_handler(
    state: web::Data<State>,
) -> actix_web::Result<web::Json<AvailableLanguagesResponse>> {
    let config = &state.config;

    let grammar_checker_langs =
        available_languages(config.data_file_dir.as_path(), DataFileType::Grammar);
    let spell_checker_langs =
        available_languages(config.data_file_dir.as_path(), DataFileType::Spelling);
    //let hyphenation_langs =
    //  available_languages(config.data_file_dir.as_path(), DataFileType::Hyphenation);

    Ok(web::Json(AvailableLanguagesResponse {
        available: AvailableLanguagesByType {
            grammar: grammar_checker_langs,
            speller: spell_checker_langs,
            //hyphenation: hyphenation_langs,
        },
    }))
}

pub fn get_gramcheck_preferences_handler(
    path: web::Path<String>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let prefs = &state.gramcheck_preferences;
    let language = path;

    let lock = prefs.read();
    let error_tags = match lock.get(&*language) {
        Some(s) => s,
        None => {
            return result(Ok(HttpResponse::InternalServerError().into()));
        }
    };

    result(Ok(HttpResponse::Ok().json(GramcheckPreferencesResponse {
        error_tags: error_tags.to_owned(),
    })))
}

pub fn gramchecker_handler(
    body: web::Json<GramcheckRequest>,
    path: web::Path<String>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let grammar_suggestions = &state.language_functions.grammar_suggestions;

    grammar_suggestions
        .suggestions(body.0, &path)
        .from_err()
        .map(|res| HttpResponse::Ok().json(res))
}

pub fn hyphenation_handler(
    body: web::Json<HyphenationRequest>,
    path: web::Path<String>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let hyphenation_suggestions = &state.language_functions.hyphenation_suggestions;

    hyphenation_suggestions
        .suggestions(body.0, &path)
        .from_err()
        .map(|res| HttpResponse::Ok().json(res))
}

pub fn speller_handler(
    body: web::Json<SpellerRequest>,
    path: web::Path<String>,
    state: web::Data<State>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let spelling_suggestions = &state.language_functions.spelling_suggestions;

    spelling_suggestions
        .suggestions(body.0, &path)
        .from_err()
        .map(|res| HttpResponse::Ok().json(res))
}
