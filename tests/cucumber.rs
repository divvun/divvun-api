#[macro_use]
extern crate cucumber_rust;

#[macro_use]
extern crate serde_json;

use std::path::PathBuf;
use std::{thread, time, env};

use divvun_api::config::Config;
use divvun_api::init::{init_config, init_system};
use divvun_api::language::speller::SpellerResponse;
use divvun_api::language::grammar::GramcheckOutput;
use divvun_api::server::state::ApiError;

static TEST_DATA_FILES: &'static str = "tests/resources/data_files";

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

mod api_steps {
    use divvun_api::language::speller::SpellerResponse;
    use divvun_api::language::grammar::GramcheckOutput;
    use divvun_api::server::state::ApiError;

    steps!(crate::MyWorld => {
        given "I have loaded `se` grammar and speller files" |world, _step| {
            let grammar_file = "grammar/se.zcheck";
            let mut dir_path = world.config.data_file_dir.clone();
            dir_path.push(grammar_file);
            assert_eq!(dir_path.exists(), true, "{} is not loaded", grammar_file);

            let speller_file = "spelling/se.zhfst";
            let mut dir_path = world.config.data_file_dir.clone();
            dir_path.push(speller_file);
            assert_eq!(dir_path.exists(), true, "{} is not loaded", speller_file);
        };

        when regex r"^I go to the endpoint `([^`]*)`$" |world, matches, _step| {
            let url = format!("http://{}{}", &world.config.addr, matches[1]);
            let body = reqwest::get(&url).unwrap().json().unwrap();

            world.json = body;
        };

        then "I get back a JSON object with available languages and their titles" |world, _step| {
            assert_eq!(&world.json, &json!({"available":{"grammar":{"se": "davvisámegiella"},"speller":{"se":"davvisámegiella"}}}));
        };

        when regex r"^I go to the endpoint `([^`]*)` with appropriate data$" |world, matches, _step| {
            let client = reqwest::Client::new();
            let url = format!("http://{}{}", &world.config.addr, matches[1]);

            match matches[1].as_str() {
                "/speller/se" => {
                    let response: SpellerResponse = client.post(&url).json(&json!({"word": "pákhat"})).send().unwrap().json().unwrap();
                    world.speller_response = Some(response);
                },
                "/grammar/se" => {
                    let response: GramcheckOutput = client.post(&url).json(&json!({"text": "sup  ney"})).send().unwrap().json().unwrap();
                    world.grammar_response = Some(response);
                },
                _ => {
                    panic!("Unsupported endpoint");
                },
            };
        };

        then regex r"^I get back a SpellerResponse with is_correct set to `([^`]*)` and some suggestions$" (bool) |world, is_correct, _step| {
            let response = &world.speller_response.clone().unwrap();
            assert_eq!(response.word, "pákhat");
            assert_eq!(response.is_correct, is_correct);
            assert_eq!(response.suggestions, vec![
                "pakehat".to_owned(), "ákkat".to_owned(), "páhkat".to_owned(), "bákčat".to_owned(), "bákŋat".to_owned()
                ]);
        };

        then "I get back a GrammarOutput with the right values" |world, _step| {
            let response = &world.grammar_response.clone().unwrap();
            assert_eq!(response.text, "sup  ney");
        };

        when regex r"^I go to the endpoint `([^`]*)` for not loaded language$" (String) |world, endpoint, _step| {
            let client = reqwest::Client::new();
            let url = format!("http://{}{}", &world.config.addr, endpoint);

            match endpoint.as_str() {
                "/speller/en" => {
                    let response: ApiError = client.post(&url).json(&json!({"word": "pákhat"})).send().unwrap().json().unwrap();
                    //panic!("{:?}", response);
                    world.api_error = Some(response);
                },
                _ => {
                    panic!("Unsupported endpoint");
                },
            };
        };

        then regex r"^I get back an ApiError with the message `([^`]*)`$" (String) | world, message, _step | {
            let error = &world.api_error.clone().unwrap();
            assert_eq!(error.message, message);
        };
    });
}

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
    before: &[],
    after: &[]
}
