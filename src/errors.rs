#![allow(dead_code)]

use catt_zwave::zwave::errors as zwave;
use catt_mqtt::errors as mqtt;

error_chain!(
    links {
        zwave::Error, zwave::ErrorKind, ZWave;
        mqtt::Error, mqtt::ErrorKind, Mqtt;
    }

    foreign_links {
        ::std::io::Error, IoError;
        ::toml::DecodeError, TomlDecodeError;
    }

    errors {
        ParseError(errors: Vec<::toml::ParserError>) {
            description("Error parsing TOML")
            display("Error parsing TOML: {:?}", errors)
        }
    }
);
