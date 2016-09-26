use std::collections::BTreeSet;

use std::sync::Arc;
use std::sync::mpsc::Receiver;

pub trait Value {
    type Error: ::std::error::Error;

    fn get_name(&self) -> String;

    fn get_string(&self) -> Result<String, Self::Error>;
    fn set_string(&self, &str) -> Result<(), Self::Error>;

    fn get_raw(&self) -> Result<Vec<u8>, Self::Error>;
    fn set_raw(&self, &[u8]) -> Result<(), Self::Error>;

    fn get_int(&self) -> Result<i64, Self::Error>;
    fn set_int(&self, i64) -> Result<i64, Self::Error>;

    fn get_unsigned(&self) -> Result<u64, Self::Error>;
    fn set_unsigned(&self, u64) -> Result<(), Self::Error>;

    fn get_float(&self) -> Result<f64, Self::Error>;
    fn set_float(&self, f64) -> Result<(), Self::Error>;

    fn get_bool(&self) -> Result<bool, Self::Error>;
    fn set_bool(&self, bool) -> Result<(), Self::Error>;
}

pub enum Notification<T> {
    Changed(T),
    Added(T),
    Removed(T),
}

pub trait Binding {
    type Error: ::std::error::Error;
    type Value: Value;

    fn get_values(&self) -> BTreeSet<Self::Value>;
    fn get_notifications(&self) -> Arc<Receiver<Notification<Self::Value>>>;
}
