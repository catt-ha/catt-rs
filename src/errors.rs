#![allow(dead_code)]

use catt_zwave::errors as zwave;
use catt_mqtt::errors as mqtt;
use catt_core::bridge;
use catt_core::bridge::config;

error_chain!(
    links {
        zwave::Error, zwave::ErrorKind, ZWave;
        mqtt::Error, mqtt::ErrorKind, Mqtt;
        config::Error, config::ErrorKind, Config;
        bridge::Error, bridge::ErrorKind, Bridge;
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
