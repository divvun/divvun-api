use std::fs;

mod config;
mod data_files;
mod grammar;
mod speller;
mod server;
mod state;
mod graphql;

use config::Config;
use server::start_server;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let config_file = "config.toml";

    let config = fs::read_to_string(config_file).expect(&format!("Failed to open {}", config_file));
    let config: Config =
        toml::from_str(&config).expect(&format!("Failed to convert {} to TOML", config_file));

    start_server(&config);
}
