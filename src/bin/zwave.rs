#[macro_use]
extern crate log;

extern crate catt;
extern crate env_logger;

use env_logger::LogBuilder;

use catt::manager::Manager;
use catt::config::Config;


use std::time;
use std::thread;

fn main() {
    LogBuilder::new().parse("catt=debug,info").init().unwrap();

    let config = Config::from_file("/home/josh/src/github.com/Pursuit92/catt/config.toml").unwrap();
    let mut manager = Manager::with_config(config).unwrap();

    let mut lock = manager.lock().unwrap();

    let driver = lock.get_driver();
    let devices = driver.get_devices();

    for dev in devices.iter() {
        info!("{}", dev);
    }


}
