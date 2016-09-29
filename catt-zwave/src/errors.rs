use std::string::FromUtf8Error;
use openzwave_stateful as ozw_s;
use openzwave as ozw;
use catt_core::value;

error_chain! {
    links {
        value::Error, value::ErrorKind, ValueError;
    }

    foreign_links {
        ozw_s::Error, ZWave;
        FromUtf8Error, Utf8;
        ::std::str::ParseBoolError, ParseBool;
        ::std::num::ParseIntError, ParseInt;
        ::std::num::ParseFloatError, ParseFloat;
    }

    errors {
        Unimplemented(item_name: String, value_type: ozw_s::ValueType) {
            description("unimplemented zwave value type")
            display("unimplemented zwave value type for item {}: {:?}", item_name, value_type)
        }
    }
}

impl From<ozw::Error> for Error {
    fn from(other: ozw::Error) -> Self {
        ozw_s::Error::from(other).into()
    }
}