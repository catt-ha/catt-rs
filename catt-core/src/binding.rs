use futures::BoxFuture;

use rustc_serialize::Decodable;

use tokio_core::reactor::Handle;
use tokio_core::channel::Receiver;

use item::Item;

pub enum Notification<T> {
    Changed(T),
    Added(T),
    Removed(T),
}

pub trait Binding {
    type Config: Default + Decodable;
    type Error: ::std::error::Error + Send + 'static;
    type Item: Item<Error = Self::Error> + Send + 'static + Clone;

    fn new(&Handle,
           &Self::Config)
           -> Result<(Self, Receiver<Notification<Self::Item>>), Self::Error>
        where Self: ::std::marker::Sized;

    fn get_value(&self, &str) -> BoxFuture<Option<Self::Item>, Self::Error>;
}
