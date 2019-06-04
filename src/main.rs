use std::{env, fs};

use clap::{crate_version, App, Arg, ArgMatches};
use log::info;

mod config;
mod graphql;
mod language;
mod server;

use config::Config;
use server::start_server;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let matches = App::new("divvun-api")
        .version(crate_version!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Set a custom TOML config file")
                .takes_value(true),
        )
        .get_matches();

    let config = get_config(&matches);

    start_server(&config);
}

fn get_config(matches: &ArgMatches) -> Config {
    let default_path = "config.toml";
    let divvun_env_var = "DIVVUN_API_CONFIG_PATH";

    let config_file = match env::var(divvun_env_var) {
        Ok(file) => {
            info!("Using {} from env var {} as config", file, divvun_env_var);
            file
        }
        Err(_) => match matches.value_of("config") {
            Some(file) => {
                info!("Using {} supplied by the CLI as config", file);
                file
            }
            None => {
                info!("Using the default {}", default_path);
                default_path
            }
        }
        .to_owned(),
    };

    let config =
        fs::read_to_string(&config_file).expect(&format!("Failed to open {}", config_file));
    let config: Config =
        toml::from_str(&config).expect(&format!("Failed to convert {} to TOML", config_file));

    config
}
