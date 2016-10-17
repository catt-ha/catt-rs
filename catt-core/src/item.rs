use futures::BoxFuture;

use value::Value;

use std::collections::HashMap;

#[derive(RustcEncodable,Debug,RustcDecodable,Default)]
pub struct Meta {
    pub backend: Option<String>,
    pub value_type: Option<String>,
    pub ext: Option<HashMap<String, String>>,
}

pub trait Item {
    type Error: ::std::error::Error + Send + 'static;

    fn get_name(&self) -> String;

    fn get_meta(&self) -> Option<Meta> {
        None
    }

    fn get_value(&self) -> BoxFuture<Value, Self::Error>;
    fn set_value(&self, Value) -> BoxFuture<(), Self::Error>;
}
