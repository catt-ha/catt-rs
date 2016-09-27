use rumqtt;
error_chain!{

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
