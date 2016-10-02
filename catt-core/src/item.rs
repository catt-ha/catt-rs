use value::Value;

use std::collections::HashMap;

#[derive(RustcEncodable,Debug,RustcDecodable,Default)]
pub struct Meta {
    pub backend: Option<String>,
    pub value_type: Option<String>,
    pub ext: Option<HashMap<String, String>>,
}

pub trait Item {
    type Error: ::std::error::Error;

    fn get_name(&self) -> String;

    fn get_meta(&self) -> Option<Meta> {
        None
    }

    fn get_value(&self) -> Result<Value, Self::Error>;
    fn set_value(&self, Value) -> Result<(), Self::Error>;
}
