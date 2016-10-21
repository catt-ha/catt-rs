#![recursion_limit = "1024"]
#![feature(conservative_impl_trait)]
#![feature(question_mark)]

#[macro_use]
extern crate error_chain;

extern crate catt_core;
extern crate catt_zwave;
extern crate catt_mqtt;

#[macro_use]
extern crate log;

use catt_core::bridge;

use catt_zwave::driver::ZWave;
use catt_mqtt::mqtt::Mqtt;

extern crate rustc_serialize;
extern crate toml;

extern crate futures;
extern crate tokio_core;

use tokio_core::reactor::Core;

mod errors;

use errors::*;

pub fn zwave(cfg: &str) -> Result<()> {
    let mut reactor = Core::new().unwrap();
    let handle = reactor.handle();
    let fut = bridge::from_file::<Mqtt, ZWave>(&handle, &cfg)?;

    let res = reactor.run(fut);

    Ok(res?)
}
