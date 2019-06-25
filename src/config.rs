use serde_derive::Deserialize;

use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub addr: String,
    pub data_file_dir: Option<PathBuf>,
}
