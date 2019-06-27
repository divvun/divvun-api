#[macro_use]
extern crate cucumber_rust;

#[macro_use]
extern crate serde_json;

use std::path::PathBuf;
use std::{env, thread, time};

use divvun_api::config::Config;
use divvun_api::init::{init_config, init_system};
use divvun_api::language::grammar::GramcheckOutput;
use divvun_api::language::speller::SpellerResponse;
use divvun_api::server::state::ApiError;

mod steps;
use steps::api;
use steps::watcher;

static TEST_DATA_FILES: &'static str = "tests/resources/data_files";
static TEST_WATCHER_INTERVAL: u64 = 500;

pub struct MyWorld {
    config: Config,
    json: serde_json::Value,
    speller_response: Option<SpellerResponse>,
    grammar_response: Option<GramcheckOutput>,
    api_error: Option<ApiError>,
}

impl cucumber_rust::World for MyWorld {}

impl Default for MyWorld {
    fn default() -> MyWorld {
        let toml_config = init_config();

        let config = Config {
            addr: toml_config.addr,
            data_file_dir: PathBuf::from(TEST_DATA_FILES),
            watcher_interval_ms: TEST_WATCHER_INTERVAL,
        };

        // This function is called every time a new scenario is started
        MyWorld {
            json: json!(""),
            config: config.clone(),
            speller_response: None,
            grammar_response: None,
            api_error: None,
        }
    }
}

// A setup function to be called before everything else
fn setup() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let toml_config = init_config();

    let config = Config {
        addr: toml_config.addr,
        data_file_dir: PathBuf::from(TEST_DATA_FILES),
        watcher_interval_ms: TEST_WATCHER_INTERVAL,
    };

    std::thread::spawn(move || {
        let (_app, system) = init_system(&config);

        system.run().unwrap();
    });

    // Sleep for a bit so the server can start before tests are ran
    thread::sleep(time::Duration::from_secs(1));
}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        api::steps,
        watcher::steps,
    ],
    setup: setup, // Optional; called once before everything
    before: &[],
    after: &[]
}
