use rumqtt;
use catt_core::value;

error_chain!{
    links {
        value::Error, value::ErrorKind, ValueError;
    }

    foreign_links {
        ::std::net::AddrParseError, AddrParseError;
        ::std::io::Error, IoError;
    }

    errors {
        Mqtt(e: rumqtt::Error) {
            description("mqtt error")
            display("mqtt error: {:?}", e)
        }

        NotStarted {
            description("mqtt client not started")
            display("mqtt client not started")
        }
    }
}

impl From<rumqtt::Error> for Error {
    fn from(other: rumqtt::Error) -> Self {
        ErrorKind::Mqtt(other).into()
    }
}
