use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info, warn};
use notify::Watcher as _;
use notify::{watcher, DebouncedEvent, RecursiveMode};

use crate::language::data_files::{get_data_dir, DataFileType};
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
                                let grammar_checkers =
                                    &msg.state.language_functions.grammar_suggestions;
                                grammar_checkers.add(file_info.stem, path.to_str().unwrap());
                            } else if file_info.extension == DataFileType::Spelling.as_ext() {
                                let spellers = &msg.state.language_functions.spelling_suggestions;
                                spellers.add(file_info.stem, path.to_str().unwrap());
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
                                let grammar_checkers =
                                    &msg.state.language_functions.grammar_suggestions;

                                grammar_checkers.remove(file_info.stem);
                                grammar_checkers.add(file_info.stem, path.to_str().unwrap());
                            } else if file_info.extension == DataFileType::Spelling.as_ext() {
                                let spellers = &msg.state.language_functions.spelling_suggestions;

                                spellers.remove(file_info.stem);
                                spellers.add(file_info.stem, path.to_str().unwrap());
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
    pub extension: &'a str,
    pub stem: &'a str,
}

fn get_file_info(path: &PathBuf) -> Option<FileInfo> {
    let extension = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => {
            warn!("File `{}` has no valid extension, skipping", path.display());
            return None;
        }
    };

    let stem = match path.file_stem().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => {
            warn!("File `{}` has no valid name, skipping", path.display());
            return None;
        }
    };

    return Some(FileInfo { extension, stem });
}
