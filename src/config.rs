use serde::Deserialize;

use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub addr: String,
    pub data_file_dir: Option<PathBuf>,
    pub watcher_interval_ms: Option<u64>,
}

#[derive(Clone)]
pub struct Config {
    pub addr: String,
    pub data_file_dir: PathBuf,
    pub watcher_interval_ms: u64,
}
