use std::io::Read;
use std::fs::File;

use toml::Parser;
use toml::Decoder;
use toml::Value;

use rustc_serialize::Decodable;

error_chain! {
    foreign_links {
        ::std::io::Error, IoError;
        ::toml::Error, TomlError;
        ::toml::DecodeError, DecodeError;
    }

    errors {
        ParseError(errors: Vec<::toml::ParserError>) {
            description("Error parsing TOML")
            display("Error parsing TOML: {:?}", errors)
        }
    }
}

#[derive(Default, RustcDecodable)]
pub struct Config<B, C> {
    pub bus: Option<B>,
    pub binding: Option<C>,
}

impl<B, C> Config<B, C> {
    pub fn from_file(file_name: &str) -> Result<Self>
        where B: Default + Decodable,
              C: Default + Decodable
    {
        let mut buf = String::new();
        let mut file = match File::open(file_name) {
            Ok(f) => f,
            Err(e) => {
                warn!("failed to read config at {} with error {:?}", file_name, e);
                return Ok(Default::default());
            }
        };

        file.read_to_string(&mut buf)?;

        let mut parser = Parser::new(&buf);
        let value = match parser.parse() {
            Some(table) => Value::Table(table),
            None => return Err(ErrorKind::ParseError(parser.errors).into()),
        };

        let mut decoder = Decoder::new(value);

        Ok(Config::decode(&mut decoder)?)
    }
}
