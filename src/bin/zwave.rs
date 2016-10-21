extern crate catt;

extern crate env_logger;

#[macro_use]
extern crate log;

use catt::zwave;

#[allow(unused_variables)]
fn main() {
    env_logger::init().unwrap();

    match zwave("config.toml") {
        Ok(()) => {}
        Err(e) => error!("fatal error: {:#?}", e),
    };
}
