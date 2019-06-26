use std::{env, fs};

use clap::{crate_version, App as ClapApp, Arg, ArgMatches};
use log::info;
use actix::{Addr, SystemRunner};
use actix_web::dev::Server;

use crate::config::{Config, TomlConfig};
use crate::server::start_server;
use crate::server::state::create_state;
use crate::watcher::{Start, Watcher};

pub struct App {
    pub config: Config,
    pub server: Server,
    pub watcher: Addr<Watcher>,
}

pub fn init_config() -> TomlConfig {
    let matches = ClapApp::new("divvun-api")
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

    get_config(&matches)
}

pub fn init_system(config: &Config) -> (App, SystemRunner) {
    let system = actix::System::new("divvun-api");
    let state = create_state(&config);

    let server_state = state.clone();
    let server = start_server(server_state, &config);

    let watcher_state = state.clone();

    let addr = actix::SyncArbiter::start(1, move || Watcher);
    addr.try_send(Start {
        state: watcher_state,
    }).unwrap();

    (App {
        config: config.clone(),
        server,
        watcher: addr,
    }, system)
}

fn get_config(matches: &ArgMatches<'_>) -> TomlConfig {
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
    let config: TomlConfig =
        toml::from_str(&config).expect(&format!("Failed to convert {} to TOML", config_file));

    config
}
