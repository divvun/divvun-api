use directories::{ProjectDirs};
use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::fs;

pub enum DataFileType {
    Grammar,
    Spelling
}

impl DataFileType {
    pub fn as_ext(&self) -> &str {
        match self {
            &DataFileType::Grammar => "zcheck",
            &DataFileType::Spelling => "zhfst",
        }
    }

    pub fn as_dir(&self) -> &str {
        match self {
            &DataFileType::Grammar => "grammar",
            &DataFileType::Spelling => "spelling",
        }
    }
}

pub fn get_data_files(data_type: &DataFileType) -> std::io::Result<Vec<PathBuf>> {
    let dir = get_data_dir(data_type);
    if !dir.is_dir() {
        return Err(Error::new(ErrorKind::InvalidInput ,"Not a directory"));
    }

    let extension = data_type.as_ext();

    let mut data_files: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() && path.extension().unwrap() == extension {
            data_files.push(path.to_path_buf());
        }
    }

    Ok(data_files)
}

fn get_data_dir(data_type: &DataFileType) -> PathBuf {
    let data_dir = match ProjectDirs::from("no", "uit", "api-giellalt") {
        Some(v) => v.data_dir().to_owned(),
        None => PathBuf::from("./"),
    };

    data_dir.join(data_type.as_dir())
}
