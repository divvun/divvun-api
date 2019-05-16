use actix_web::{HttpResponse, web};

use futures::future::{result, Future};

use crate::server::state::State;

use super::data_files::{available_languages, AvailableLanguagesResponse, AvailableLanguagesByType, DataFileType};
use super::grammar::{GramcheckPreferencesResponse, GramcheckRequest};
use super::speller::SpellerRequest;

pub fn get_available_languages_handler() -> actix_web::Result<web::Json<AvailableLanguagesResponse>> {
    let grammar_checker_langs = available_languages(DataFileType::Grammar);
    let spell_checker_langs = available_languages(DataFileType::Spelling);

    Ok(web::Json(AvailableLanguagesResponse {
        available: AvailableLanguagesByType {
            grammar: grammar_checker_langs,
            speller: spell_checker_langs,
        },
    }))
}

pub fn get_gramcheck_preferences_handler(
    path: web::Path<String>,
    state: web::Data<State>)
-> impl Future<Item=HttpResponse, Error=actix_web::Error> {

    let prefs = &state.gramcheck_preferences;
    let language = path;

    let error_tags = match prefs.get(&*language) {
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
    state: web::Data<State>)
-> impl Future<Item=HttpResponse, Error=actix_web::Error> {

    let grammar_suggestions = &state.language_functions.grammar_suggesgions;

    grammar_suggestions.grammar_suggestions(body.0, &path)
        .from_err()
        .and_then(|res| match res {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
}

pub fn speller_handler(
    body: web::Json<SpellerRequest>,
    path: web::Path<String>,
    state: web::Data<State>)
    -> impl Future<Item=HttpResponse, Error=actix_web::Error> {
    let spelling_suggestions = &state.language_functions.spelling_suggestions;

    spelling_suggestions.spelling_suggestions(body.0, &path)
        .from_err()
        .and_then(|res| match res {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
}
