use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

pub enum ValueType {}

pub trait Value {
    type Error: ::std::error::Error;

    fn get_name(&self) -> String;

    fn get_value(&self) -> Result<Vec<u8>, Self::Error>;
    fn set_value(&self, &[u8]) -> Result<(), Self::Error>;
}

pub enum Notification<T> {
    Changed(T),
    Added(T),
    Removed(T),
}

pub trait Binding {
    type Error: ::std::error::Error;
    type Value: Value + Send + 'static + Clone;

    fn get_values(&self) -> BTreeMap<String, Self::Value>;
    fn notifications(&self) -> Arc<Mutex<Receiver<Notification<Self::Value>>>>;
}
