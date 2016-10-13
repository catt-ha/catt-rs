extern crate catt;

extern crate env_logger;

use catt::zwave;

#[allow(unused_variables)]
fn main() {
    env_logger::init().unwrap();

    let _ = zwave("config.toml");
}
