use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;

use value::Value;
use item::Meta;

#[derive(Debug)]
pub enum Message {
    Update(String, Value),
    Command(String, Value),
    Meta(String, Meta),
}

#[derive(Debug)]
pub enum SubType {
    Update,
    Command,
    Meta,
    All,
}

pub trait Bus {
    type Error: ::std::error::Error;

    fn publish(&self, Message) -> Result<(), Self::Error>;
    fn subscribe(&self, item_name: &str, SubType) -> Result<(), Self::Error>;
    fn unsubscribe(&self, item_name: &str, SubType) -> Result<(), Self::Error>;

    fn messages(&self) -> Arc<Mutex<Receiver<Message>>>;
}
