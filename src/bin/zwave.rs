extern crate catt;

extern crate env_logger;

use env_logger::LogBuilder;

use catt::init;

#[allow(unused_variables)]
fn main() {
    env_logger::init().unwrap();

    let bridge = init("config.toml").unwrap();

    bridge.join_all();
}
