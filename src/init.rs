use std::{env, fs};

use clap::{crate_version, App, Arg, ArgMatches};
use log::info;

use crate::config::Config;
use crate::server::start_server;
use crate::server::state::create_state;
use crate::watcher::{Start, Watcher};

pub fn init() {
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

    let system = actix::System::new("divvun-api");
    let state = create_state();

    let server_state = state.clone();
    let server = start_server(server_state, &config);

    let watcher_state = state.clone();
    let addr = actix::SyncArbiter::start(1, move || Watcher);
    addr.try_send(Start {
        state: watcher_state,
    })
    .unwrap();

    system.run().unwrap();
}

pub fn shut_down(system: actix::System, server: actix_web::dev::Server) {
    system.stop();
}

fn get_config(matches: &ArgMatches<'_>) -> Config {
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
