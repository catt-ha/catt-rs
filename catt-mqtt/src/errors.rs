use mqttc;
use mqtt3;

error_chain!{

    foreign_links {
        ::std::net::AddrParseError, AddrParseError;
        ::std::io::Error, IoError;
    }

    errors {
        Mqttc(e: mqttc::Error) {
            description("Mqtt client error")
            display("Mqtt client error: {:?}", e)
        }
        Mqtt3(e: mqtt3::Error) {
            description("Mqtt proto error")
                display("Mqtt proto error: {:?}", e)
        }
    }
}

impl From<mqttc::Error> for Error {
    fn from(other: mqttc::Error) -> Self {
        ErrorKind::Mqttc(other).into()
    }
}

impl From<mqtt3::Error> for Error {
    fn from(other: mqtt3::Error) -> Self {
        ErrorKind::Mqtt3(other).into()
    }
}
