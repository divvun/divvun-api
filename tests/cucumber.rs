#[macro_use]
extern crate cucumber_rust;

#[macro_use]
extern crate serde_json;

use std::{thread, time};

pub struct MyWorld {
    // You can use this struct for mutable context in scenarios.
    json: serde_json::Value,
}

impl cucumber_rust::World for MyWorld {}

impl Default for MyWorld {
    fn default() -> MyWorld {
        // This function is called every time a new scenario is started
        MyWorld {
            json: json!(""),
        }
    }
}

mod example_steps {
    // Any type that implements cucumber_rust::World + Default can be the world
    steps!(crate::MyWorld => {
        given "I have loaded `se` grammar and speller files" |_world, _step| { };

        when "I go to the endpoint `/languages`" |world, _step| {
            // TODO: pull from toml
            let body = reqwest::get("http://127.0.0.1:8080/languages").unwrap().json().unwrap();

            world.json = body;
        };

        then "I get back a JSON object with available languages and their titles" |world, _step| {
            assert_eq!(&world.json, &json!({"available":{"grammar":{"se": "davvisámegiella"},"speller":{"se":"davvisámegiella"}}}));
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
    use divvun_api::init::init;

    thread::spawn(move || {
        init();
    });

    // Sleep for a bit so the server can start before tests are ran
    thread::sleep(time::Duration::from_secs(3));
}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        example_steps::steps // the `steps!` macro creates a `steps` function in a module
    ],
    setup: setup, // Optional; called once before everything
    before: &[
        a_before_fn // Optional; called before each scenario
    ],
    after: &[
        an_after_fn // Optional; called after each scenario
    ]
}
