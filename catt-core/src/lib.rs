#![feature(question_mark)]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;

#[macro_use]
extern crate rustc_serialize;

extern crate byteorder;

extern crate toml;

pub mod util;
pub mod bus;
pub mod value;
pub mod item;
pub mod binding;
pub mod bridge;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
