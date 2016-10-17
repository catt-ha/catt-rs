use rustc_serialize::Decodable;

use tokio_core::reactor::Handle;
use tokio_core::channel::Receiver;

use futures::BoxFuture;

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
    type Config: Default + Decodable;
    type Error: ::std::error::Error + Send + 'static;

    fn new(&Handle, &Self::Config) -> Result<(Self, Receiver<Message>), Self::Error>
        where Self: ::std::marker::Sized;

    fn publish(&self, Message) -> BoxFuture<(), Self::Error>;
    fn subscribe(&self, item_name: &str, SubType) -> BoxFuture<(), Self::Error>;
    fn unsubscribe(&self, item_name: &str, SubType) -> BoxFuture<(), Self::Error>;
}
