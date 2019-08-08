use std::path::PathBuf;

use log::{error, warn};

pub struct FileInfo<'a> {
    pub path: &'a str,
    pub extension: &'a str,
    pub stem: &'a str,
}

pub fn get_file_info(original_path: &PathBuf) -> Option<FileInfo<'_>> {
    let path = match original_path.to_str() {
        Some(path) => path,
        None => {
            error!(
                "Path failed to convert to string: {}",
                original_path.display()
            );
            return None;
        }
    };

    let extension = match original_path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => {
            warn!(
                "File `{}` has no valid extension, skipping",
                original_path.display()
            );
            return None;
        }
    };

    let stem = match original_path.file_stem().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => {
            warn!(
                "File `{}` has no valid name, skipping",
                original_path.display()
            );
            return None;
        }
    };

    return Some(FileInfo {
        path,
        extension,
        stem,
    });
}
