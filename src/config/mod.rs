mod errors;

use rustc_serialize::Decodable;

pub use self::errors::*;

use std::fs::File;
use std::io::Read;

use toml::Decoder;
use toml::Parser;
use toml::Value;

pub const MQTT_BASE_DEFAULT: &'static str = "catt/items";
pub const MQTT_QOS_DEFAULT: u8 = 0;

#[derive(RustcDecodable,Debug,Clone)]
pub struct Config {
    pub mqtt: BrokerConfig,
    pub zwave: ZWaveConfig,
}

#[derive(RustcDecodable,Debug,Clone)]
pub struct BrokerConfig {
    pub broker: String,
    pub item_base: Option<String>,
    pub client_id: Option<String>,
    pub qos: Option<u8>,
    pub tls: Option<bool>,
}

#[derive(RustcDecodable,Debug,Clone)]
pub struct ZWaveConfig {
    pub port: Option<String>,
    pub sys_config: Option<String>,
    pub user_config: Option<String>,
    pub device: Vec<DeviceConfig>,
    pub expose_unbound: bool,
}

#[derive(RustcDecodable,Debug,Clone)]
pub struct DeviceConfig {
    pub name: String,
    pub id: u64,
    pub command_class: Option<String>,
    pub genre: Option<String>,
    pub value_type: Option<String>,
    pub index: Option<u8>,
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
