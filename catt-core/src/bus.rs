use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub item_name: String,
    pub value: Vec<u8>,
}

#[derive(Debug)]
pub enum MessageType {
    Update,
    Command,
}

#[derive(Debug)]
pub enum SubType {
    Update,
    Command,
    All,
}

pub trait Bus {
    type Error: ::std::error::Error;

    fn publish(&self, Message) -> Result<(), Self::Error>;
    fn subscribe(&self, item_name: &str, SubType) -> Result<(), Self::Error>;

    fn messages(&self) -> Arc<Mutex<Receiver<Message>>>;
}
