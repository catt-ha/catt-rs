extern crate catt;

extern crate env_logger;

use env_logger::LogBuilder;

use catt::init;

#[allow(unused_variables)]
fn main() {
    let _ = LogBuilder::new().parse("catt_core=debug,catt_zwave=debug,catt_mqtt=debug").init();

    let bridge = init("config.toml").unwrap();

    bridge.join_all();
}
