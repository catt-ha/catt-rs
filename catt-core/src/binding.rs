use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

use item::Item;

pub enum Notification<T> {
    Changed(T),
    Added(T),
    Removed(T),
}

pub trait Binding {
    type Config;
    type Error: ::std::error::Error + Send + 'static;
    type Item: Item + Send + 'static + Clone;

    fn new(&Self::Config) -> Result<(Self, Receiver<Notification<Self::Item>>), Self::Error>
        where Self: ::std::marker::Sized;

    fn get_values(&self) -> Arc<Mutex<BTreeMap<String, Self::Item>>>;
}
