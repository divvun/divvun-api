use std::sync::mpsc::channel;
use std::time::Duration;

use actix::prelude::*;
use log::{error, info};
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
        watcher.watch(&dir, RecursiveMode::NonRecursive).unwrap();

        info!("Watching directory {} for grammar files", dir.display());

        loop {
            match rx.recv() {
                Ok(event) => match &event {
                    DebouncedEvent::Create(path) => {
                        let extension = path.extension().unwrap().to_str().unwrap();

                        if extension == DataFileType::Grammar.as_ext() {
                            let file_stem = path.file_stem().unwrap().to_str().unwrap();

                            let grammar_checkers =
                                &msg.state.language_functions.grammar_suggestions;
                            grammar_checkers.add(file_stem, path.into());
                        }

                        info!("Create Event {:?} for file {}", &event, &extension)
                    }
                    DebouncedEvent::Remove(path) => {
                        let extension = path.extension().unwrap().to_str().unwrap();

                        if extension == DataFileType::Grammar.as_ext() {
                            let file_stem = path.file_stem().unwrap().to_str().unwrap();

                            let grammar_checkers =
                                &msg.state.language_functions.grammar_suggestions;
                            grammar_checkers.remove(file_stem);
                        }

                        info!("Create Event {:?} for file {}", &event, &extension)
                    }
                    _ => info!("{:?}", &event),
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    }
}
