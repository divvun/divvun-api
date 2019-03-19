use directories::ProjectDirs;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::collections::HashMap;
use csv;

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

pub fn available_languages() -> HashMap<String, String> {
    let autonyms_tsv = include_str!("../assets/iso639-autonyms.tsv");
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(autonyms_tsv.as_bytes());

    let mut result = HashMap::new();

    for record in reader.records() {
        if let Ok(record) = record {
            let iso_639_1_code: String = match record.get(1) {
                Some(v) => v.into(),
                None => "".into()
            };
            let autonym: String = match record.get(3) {
                Some(v) => v.into(),
                None => "<missing name>".into(),
            };
            result.insert(iso_639_1_code, autonym);
        }
    }

    result
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
