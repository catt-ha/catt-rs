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
    type Error: ::std::error::Error;
    type Item: Item + Send + 'static + Clone;

    fn get_values(&self) -> Arc<Mutex<BTreeMap<String, Self::Item>>>;
    fn notifications(&self) -> Arc<Mutex<Receiver<Notification<Self::Item>>>>;
}
