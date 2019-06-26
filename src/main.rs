use divvun_api::init::{init_config, init_system};

use std::env;
use std::path::PathBuf;

use directories::ProjectDirs;
use divvun_api::config::Config;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let toml_config = init_config();

    let config = Config {
        addr: toml_config.addr,
        data_file_dir: match toml_config.data_file_dir {
            Some(dir) => dir,
            None => match ProjectDirs::from("no", "uit", "api-giellalt") {
                Some(v) => v.data_dir().to_owned(),
                None => PathBuf::from("./"),
            },
        },
    };

    let (_app, system) = init_system(&config);

    system.run().unwrap();
}
