use divvun_api::init::{init_config, init_system};

fn main() {
    let config = init_config();

    init_system(&config);
}
