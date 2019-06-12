use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info, warn};
use notify::Watcher as _;
use notify::{watcher, DebouncedEvent, RecursiveMode};

use crate::language::data_files::{get_data_dir, DataFileType};
use crate::language::grammar::list_preferences;
use crate::server::state::State;

pub struct Watcher;

impl Actor for Watcher {
    type Context = SyncContext<Self>;
}

pub struct Start {
    pub state: State,
}

impl Message for Start {
    type Result = Result<(), ()>;
}

impl Handler<Start> for Watcher {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: Start, _: &mut Self::Context) -> Self::Result {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        let dir = get_data_dir(DataFileType::Grammar);
        watcher
            .watch(
                get_data_dir(DataFileType::Grammar),
                RecursiveMode::NonRecursive,
            )
            .unwrap();
        info!("Watching directory `{}` for grammar files", dir.display());

        let dir = get_data_dir(DataFileType::Spelling);
        watcher
            .watch(
                get_data_dir(DataFileType::Spelling),
                RecursiveMode::NonRecursive,
            )
            .unwrap();
        info!("Watching directory `{}` for speller files", dir.display());

        loop {
            match rx.recv() {
                Ok(event) => match &event {
                    DebouncedEvent::Create(path) => {
                        info!("Event {:?}", &event);

                        if let Some(file_info) = get_file_info(path) {
                            if file_info.extension == DataFileType::Grammar.as_ext() {
                                let preferences = match list_preferences(file_info.path) {
                                    Ok(preferences) => preferences,
                                    Err(e) => {
                                        error!("Failed to retrieve grammar preferences for {}: {}, ignoring file", e, file_info.stem);
                                        continue;
                                    }
                                };

                                let grammar_checkers =
                                    &msg.state.language_functions.grammar_suggestions;
                                grammar_checkers.add(file_info.stem, file_info.path);

                                let prefs_lock = &mut msg.state.gramcheck_preferences.write();
                                prefs_lock.insert(file_info.stem.to_owned(), preferences);
                            } else if file_info.extension == DataFileType::Spelling.as_ext() {
                                let spellers = &msg.state.language_functions.spelling_suggestions;
                                spellers.add(file_info.stem, file_info.path);
                            }
                        }
                    }
                    DebouncedEvent::Remove(path) => {
                        info!("Event {:?}", &event);

                        if let Some(file_info) = get_file_info(path) {
                            if file_info.extension == DataFileType::Grammar.as_ext() {
                                let grammar_checkers =
                                    &msg.state.language_functions.grammar_suggestions;
                                grammar_checkers.remove(file_info.stem);

                                let prefs_lock = &mut msg.state.gramcheck_preferences.write();
                                prefs_lock.remove(file_info.stem);
                            } else if file_info.extension == DataFileType::Spelling.as_ext() {
                                let spellers = &msg.state.language_functions.spelling_suggestions;
                                spellers.remove(file_info.stem);
                            }
                        }
                    }
                    DebouncedEvent::Write(path) => {
                        info!("Event {:?}", &event);

                        if let Some(file_info) = get_file_info(path) {
                            if file_info.extension == DataFileType::Grammar.as_ext() {
                                let preferences = match list_preferences(file_info.path) {
                                    Ok(preferences) => preferences,
                                    Err(e) => {
                                        error!("Failed to retrieve grammar preferences for {}: {}, ignoring file", e, file_info.stem);
                                        continue;
                                    }
                                };

                                let grammar_checkers =
                                    &msg.state.language_functions.grammar_suggestions;

                                grammar_checkers.remove(file_info.stem);
                                grammar_checkers.add(file_info.stem, file_info.path);

                                let prefs_lock = &mut msg.state.gramcheck_preferences.write();
                                prefs_lock.remove(file_info.stem);
                                prefs_lock.insert(file_info.stem.to_owned(), preferences);
                            } else if file_info.extension == DataFileType::Spelling.as_ext() {
                                let spellers = &msg.state.language_functions.spelling_suggestions;

                                spellers.remove(file_info.stem);
                                spellers.add(file_info.stem, file_info.path);
                            }
                        }
                    }
                    _ => info!("Event {:?}", &event),
                },
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    }
}

struct FileInfo<'a> {
    pub path: &'a str,
    pub extension: &'a str,
    pub stem: &'a str,
}

fn get_file_info(original_path: &PathBuf) -> Option<FileInfo<'_>> {
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
