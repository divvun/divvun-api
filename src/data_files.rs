use actix_web::{HttpRequest, Json};
use csv;
use directories::ProjectDirs;
use serde_derive::Serialize;

use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use crate::server::State;

#[derive(Clone, Copy)]
pub enum DataFileType {
    Grammar,
    Spelling,
}

impl DataFileType {
    pub fn as_ext(&self) -> &str {
        match self {
            DataFileType::Grammar => "zcheck",
            DataFileType::Spelling => "zhfst",
        }
    }

    pub fn as_dir(&self) -> &str {
        match self {
            DataFileType::Grammar => "grammar",
            DataFileType::Spelling => "spelling",
        }
    }
}

pub fn available_languages(data_type: DataFileType) -> HashMap<String, String> {
    let autonyms_tsv = include_str!("../assets/iso639-autonyms.tsv");
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(autonyms_tsv.as_bytes());

    let lang_data_files = match get_data_files(data_type) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };

    let lang_keys: Vec<String> = lang_data_files
        .iter()
        .map(|p| {
            p.file_stem()
                .expect("Somehow this doesn't have a filestem")
                .to_str()
                .expect("Somehow this OsStr cannot be converted to str")
                .to_owned()
        })
        .collect();

    let all_langs: HashMap<String, String> = reader
        .records()
        .filter_map(|r| r.ok())
        .filter(|r| r.get(1).is_some())
        .filter(|r| r.get(3).is_some())
        .map(|r| (r.get(1).unwrap().to_owned(), r.get(3).unwrap().to_owned()))
        .collect();

    let result: HashMap<String, String> = lang_keys
        .iter()
        .map(|k| (k.clone(), all_langs[k].clone()))
        .collect();

    result
}

#[derive(Serialize)]
struct AvailableLanguagesByType {
    grammar: HashMap<String, String>,
    speller: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct AvailableLanguagesResponse {
    available: AvailableLanguagesByType,
}

pub fn get_available_languages(
    _req: &HttpRequest<State>,
) -> actix_web::Result<Json<AvailableLanguagesResponse>> {
    let grammar_checker_langs = available_languages(DataFileType::Grammar);
    let spell_checker_langs = available_languages(DataFileType::Spelling);

    Ok(Json(AvailableLanguagesResponse {
        available: AvailableLanguagesByType {
            grammar: grammar_checker_langs,
            speller: spell_checker_langs,
        },
    }))
}

pub fn get_data_files(data_type: DataFileType) -> std::io::Result<Vec<PathBuf>> {
    let dir = get_data_dir(data_type);

    let extension = data_type.as_ext();

    let paths = fs::read_dir(dir)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Not a directory"))?
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|path| !path.is_dir())
        .filter(|path| path.extension().unwrap_or_default() == extension)
        .collect();

    Ok(paths)
}

fn get_data_dir(data_type: DataFileType) -> PathBuf {
    let data_dir = match ProjectDirs::from("no", "uit", "api-giellalt") {
        Some(v) => v.data_dir().to_owned(),
        None => PathBuf::from("./"),
    };

    data_dir.join(data_type.as_dir())
}
