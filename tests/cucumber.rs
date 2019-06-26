#[macro_use]
extern crate cucumber_rust;

#[macro_use]
extern crate serde_json;

use divvun_api::config::Config;
use divvun_api::init::{init_config, init_system};

use std::env;
use std::path::PathBuf;
use std::{thread, time};

use divvun_api::language::speller::SpellerResponse;

static TEST_DATA_FILES: &'static str = "tests/resources/data_files";

pub struct MyWorld {
    config: Config,
    json: serde_json::Value,
    speller_response: Option<SpellerResponse>,
}

impl cucumber_rust::World for MyWorld {}

impl Default for MyWorld {
    fn default() -> MyWorld {
        let toml_config = init_config();

        let config = Config {
            addr: toml_config.addr,
            data_file_dir: PathBuf::from(TEST_DATA_FILES),
        };

        // This function is called every time a new scenario is started
        MyWorld {
            json: json!(""),
            config: config.clone(),
            speller_response: None,
        }
    }
}

mod api_steps {
    use divvun_api::language::speller::SpellerResponse;

    steps!(crate::MyWorld => {
        given "I have loaded `se` grammar and speller files" |_world, _step| {

        };

        when regex r"^I go to the endpoint `([^`]*)`$" |world, matches, _step| {
            let url = format!("http://{}{}", &world.config.addr, matches[1]);
            let body = reqwest::get(&url).unwrap().json().unwrap();

            world.json = body;
        };

        then "I get back a JSON object with available languages and their titles" |world, _step| {
            assert_eq!(&world.json, &json!({"available":{"grammar":{"se": "davvisámegiella"},"speller":{"se":"davvisámegiella"}}}));
        };

        when regex r"^I go to the endpoint `([^`]*)` with data$" |world, matches, _step| {
            let url = format!("http://{}{}", &world.config.addr, matches[1]);

            let client = reqwest::Client::new();

            let response: SpellerResponse = client.post(&url).json(&json!({"word": "pákhat"})).send().unwrap().json().unwrap();
            world.speller_response = Some(response);
        };

        then regex r"^I get back a JSON object with is_correct set to `([^`]*)` and some suggestions$" (bool) |world, is_correct, _step| {
            let response = &world.speller_response.clone().unwrap();
            assert_eq!(response.word, "pákhat");
            assert_eq!(response.is_correct, is_correct);
            assert_eq!(response.suggestions, vec![
                "pakehat".to_owned(), "ákkat".to_owned(), "páhkat".to_owned(), "bákčat".to_owned(), "bákŋat".to_owned()
                ]);
        };
    });
}

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {

});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
fn setup() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let toml_config = init_config();

    let config = Config {
        addr: toml_config.addr,
        data_file_dir: PathBuf::from(TEST_DATA_FILES),
    };

    init_system(&config);

    // Sleep for a bit so the server can start before tests are ran
    thread::sleep(time::Duration::from_secs(2));
}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        api_steps::steps // the `steps!` macro creates a `steps` function in a module
    ],
    setup: setup, // Optional; called once before everything
    before: &[
        a_before_fn // Optional; called before each scenario
    ],
    after: &[
        an_after_fn // Optional; called after each scenario
    ]
}
