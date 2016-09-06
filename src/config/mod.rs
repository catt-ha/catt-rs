mod errors;

use rustc_serialize::Decodable;

pub use self::errors::*;

use std::fs::File;
use std::io::Read;

use toml::Decoder;
use toml::Parser;
use toml::Value;

#[derive(RustcDecodable,Debug)]
pub struct Config {
    pub mqtt: BrokerConfig,
    pub zwave: ZWaveConfig,
}

#[derive(RustcDecodable,Debug)]
pub struct BrokerConfig {
    pub broker: String,
    pub client_id: Option<String>,
}

#[derive(RustcDecodable,Debug)]
pub struct ZWaveConfig {
    pub port: String,
    pub device: Vec<DeviceConfig>,
}

#[derive(RustcDecodable,Debug)]
pub struct DeviceConfig {
    pub name: String,
    pub id: u64,
    pub command: Option<String>,
    pub endpoint: Option<u64>,
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


#[cfg(test)]
mod test {
    use super::Config;

    #[test]
    fn deserialize_config() {
        let config = Config::from_file("config.toml").unwrap();
        println!("{:?}", config);
    }
}
