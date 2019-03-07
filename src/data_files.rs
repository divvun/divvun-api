use directories::ProjectDirs;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

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
