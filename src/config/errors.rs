#![allow(dead_code)]

error_chain!(
    links {
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
