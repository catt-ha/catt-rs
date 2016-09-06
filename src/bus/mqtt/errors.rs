use mqtt3;

error_chain!{

    foreign_links {
        ::std::net::AddrParseError, AddrParseError;
        ::std::io::Error, IoError;
    }

    errors {
        MqttError(e: mqtt3::Error) {
            description("Mqtt protocol error")
            display("mqtt error: {:?}", e)
        }
    }
}

impl From<mqtt3::Error> for Error {
    fn from(other: mqtt3::Error) -> Self {
        ErrorKind::MqttError(other).into()
    }
}
