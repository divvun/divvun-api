use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use log::warn;
use serde::Serialize;

include!(concat!(env!("OUT_DIR"), "/autonyms.rs"));

#[derive(Clone, Copy)]
pub enum DataFileType {
    Grammar,
    Spelling,
    Hyphenation,
}

#[derive(Serialize)]
pub struct AvailableLanguagesByType {
    pub grammar: HashMap<String, String>,
    pub speller: HashMap<String, String>,
    pub hyphenation: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct AvailableLanguagesResponse {
    pub available: AvailableLanguagesByType,
}

impl DataFileType {
    pub fn as_ext(&self) -> &str {
        match self {
            DataFileType::Grammar => "zcheck",
            DataFileType::Spelling => "zhfst",
            DataFileType::Hyphenation => "hfstol",
        }
    }

    pub fn as_dir(&self) -> &str {
        match self {
            DataFileType::Grammar => "grammar",
            DataFileType::Spelling => "spelling",
            DataFileType::Hyphenation => "hyphenation",
        }
    }
}

#[derive(Debug)]
struct Record {
    tag3: &'static str,
    tag1: Option<&'static str>,
    name: Option<&'static str>,
    autonym: Option<&'static str>,
    source: Option<&'static str>,
}

pub fn available_languages(
    data_file_dir: &Path,
    data_type: DataFileType,
) -> HashMap<String, String> {
    let lang_data_files = match get_data_files(data_file_dir, data_type) {
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

    let result: HashMap<String, String> = lang_keys
        .iter()
        .map(|k| {
            if !LANGUAGE_AUTONYMS.contains_key::<str>(k) {
                warn!("Key {} not found in autonyms file", k);
                (k.clone(), k.clone())
            } else {
                let record = LANGUAGE_AUTONYMS.get::<str>(k).unwrap();

                let value = record
                    .autonym
                    .or(record.name)
                    .or(Some(record.tag3))
                    .unwrap();

                (k.clone(), value.to_owned())
            }
        })
        .collect();

    result
}

pub fn get_data_files(
    data_file_dir: &Path,
    data_type: DataFileType,
) -> std::io::Result<Vec<PathBuf>> {
    let dir = get_typed_data_dir(data_file_dir, data_type);

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

pub fn get_typed_data_dir(data_file_dir: &Path, data_type: DataFileType) -> PathBuf {
    data_file_dir.join(data_type.as_dir())
}
