#![feature(question_mark)]

#[cfg(test)]
extern crate env_logger;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;

extern crate rumqtt;

extern crate rustc_serialize;

extern crate catt_core;

pub mod errors;
pub mod config;

pub mod mqtt;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
