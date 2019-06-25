#[macro_use]
extern crate cucumber_rust;

#[macro_use]
extern crate serde_json;

use divvun_api::config::Config;
use divvun_api::init::{init_config, init_system};

use std::fs;
use std::path::PathBuf;
use std::{thread, time};

static TEST_DATA_FILES: &'static str = "tests/resources/data_files";

pub struct MyWorld {
    config: Config,
    json: serde_json::Value,
}

impl cucumber_rust::World for MyWorld {}

impl Default for MyWorld {
    fn default() -> MyWorld {
        let toml_config = init_config();

        let config = Config {
            addr: toml_config.addr,
            data_file_dir: PathBuf::from(TEST_DATA_FILES),
        };

        let config_clone = config.clone();

        thread::spawn(move || {
            init_system(&config);
        });

        // Sleep for a bit so the server can start before tests are ran
        thread::sleep(time::Duration::from_secs(1));

        // This function is called every time a new scenario is started
        MyWorld {
            json: json!(""),
            config: config_clone,
        }
    }
}

mod api_steps {
    use std::fs;

    steps!(crate::MyWorld => {
        given "I have loaded `se` grammar and speller files" |world, _step| {
            let data_file_dir = &world.config.data_file_dir;

            fs::copy(format!("{}/se.zhfst", data_file_dir.display()),
                format!("{}/spelling/se.zhfst", data_file_dir.display())).unwrap();

            fs::copy(format!("{}/se.zcheck", data_file_dir.display()),
                format!("{}/grammar/se.zcheck", data_file_dir.display())).unwrap();
        };

        when regex "I go to the endpoint `(.*)`" |world, matches, _step| {
            let url = format!("http://{}{}", &world.config.addr, matches[1]);
            let body = reqwest::get(&url).unwrap().json().unwrap();

            world.json = body;
        };

        then "I get back a JSON object with available languages and their titles" |world, _step| {
            assert_eq!(&world.json, &json!({"available":{"grammar":{"se": "davvisámegiella"},"speller":{"se":"davvisámegiella"}}}));

            let data_file_dir = &world.config.data_file_dir;

            fs::remove_file(format!("{}/spelling/se.zhfst", data_file_dir.display())).unwrap();
            fs::remove_file(format!("{}/grammar/se.zcheck", data_file_dir.display())).unwrap();
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
    match fs::metadata(format!("{}/se.zhfst", TEST_DATA_FILES)) {
        Ok(_) => {}
        Err(_e) => {
            panic!(
                "No file found at {}, make sure `se` files are placed in the {} folder",
                format!("{}/se.zhfst", TEST_DATA_FILES),
                TEST_DATA_FILES
            );
        }
    };

    match fs::metadata(format!("{}/se.zcheck", TEST_DATA_FILES)) {
        Ok(_) => (),
        Err(_e) => {
            panic!(
                "No file found at {}, make sure `se` files are placed in the {} folder",
                format!("{}/se.zcheck", TEST_DATA_FILES),
                TEST_DATA_FILES
            );
        }
    };
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
