use std::fs::File;

use std::io::Read;

use toml::Parser;
use toml::Table;
use toml::Value;
use toml::Decoder;

use rustc_serialize::Decodable;

use errors::*;

#[derive(RustcDecodable)]
pub struct Config {
    pub mqtt: ::catt_mqtt::config::Config,
    pub zwave: ::catt_zwave::config::ZWaveConfig,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Self> {
        read_config(file_name)
    }
}

fn read_config(file_name: &str) -> Result<Config> {
    let mut buf = String::new();
    let mut file = File::open(file_name)?;

    file.read_to_string(&mut buf)?;

    let mut parser = Parser::new(&buf);
    let value = match parser.parse() {
        Some(table) => Value::Table(table),
        None => return Err(ErrorKind::ParseError(parser.errors).into()),
    };

    let mut decoder = Decoder::new(value);

    Ok(Config::decode(&mut decoder)?)
}
