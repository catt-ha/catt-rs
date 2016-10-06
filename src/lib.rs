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

#[macro_use]
extern crate log;

use catt_core::bridge::Bridge;
use catt_core::bridge::Config;
use catt_mqtt::mqtt::Mqtt;
use catt_zwave::driver::ZWave;

extern crate rustc_serialize;
extern crate toml;

mod config;
mod errors;

use errors::*;

pub fn init(config_file: &str) -> Result<Bridge<Mqtt, ZWave>> {
    let cfg = Config::from_file(config_file)?;
    Ok(Bridge::new(cfg)?)
}
