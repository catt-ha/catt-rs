#![recursion_limit = "1024"]
#![feature(conservative_impl_trait)]
#![feature(question_mark)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate error_chain;

extern crate catt_core;
extern crate catt_zwave;
extern crate catt_mqtt;

use catt_core::bridge::Bridge;
use catt_mqtt::mqtt::MqttBus;
use catt_zwave::zwave::driver::ZWave;

extern crate rustc_serialize;
extern crate toml;

mod config;
mod errors;

use errors::*;

pub fn init(config_file: &str) -> Result<Bridge<MqttBus, ZWave>> {
    let cfg = config::Config::from_file(config_file)?;
    Ok(Bridge::new(MqttBus::with_config(&cfg.mqtt)?, ZWave::new(&cfg.zwave)?))
}
