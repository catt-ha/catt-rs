use rustc_serialize::Decodable;

use tokio_core::reactor::Handle;
use futures::stream::Stream;
use futures::Future;

use binding::Binding;
use binding::Notification;

pub mod config;
pub use self::config::Config;

use bus::Bus;
use bus::SubType;
use bus::Message;

use item::Item;
use item::Meta;

use std::error::Error as SError;

error_chain! {
    links {
        config::Error, config::ErrorKind, ConfigError;
    }

    foreign_links {
        ::std::io::Error, IoError;
    }

    errors {
        Bus(e: Box<SError + Send + 'static>) {
            display("bus error: {}", e)
            description("bus error")
        }
        Binding(e: Box<SError + Send + 'static>) {
            display("binding error: {}", e)
            description("binding error")
        }
    }
}

pub fn new<B, C>(handle: &Handle, cfg: Config<B::Config, C::Config>) -> Result<(impl Future<Item=(), Error=Error>, impl Future<Item=(), Error=Error>)>
    where B: Bus,
          C: Binding,
          B::Config: Default,
          C::Config: Default
{

    let (bus, messages) = match B::new(handle, &cfg.bus.unwrap_or_default()) {
        Ok(b) => b,
        Err(e) => return Err(ErrorKind::Bus(Box::new(e)).into()),
    };
    let (binding, notifications) = match C::new(handle, &cfg.binding.unwrap_or_default()) {
        Ok(b) => b,
        Err(e) => return Err(ErrorKind::Binding(Box::new(e)).into()),
    };

    let msg_fut = messages
        .map_err(Error::from)
        .for_each(bus_to_binding(binding));
    let not_fut = notifications
        .map_err(Error::from)
        .for_each(binding_to_bus(bus));

    Ok((msg_fut, not_fut))
}

pub fn from_file<B, C>(handle: &Handle, config_file: &str) -> Result<(impl Future<Item=(), Error=Error>, impl Future<Item=(), Error=Error>)>
    where B: Bus,
          C: Binding,
          B::Config: Default + Decodable,
          C::Config: Default + Decodable,

{

    let cfg: Config<B::Config, C::Config> = Config::from_file(config_file)?;
    Ok(new::<B, C>(handle, cfg)?)
}

fn bus_to_binding<C>(binding: C) -> impl FnMut(Message) -> Result<()>
    where C: Binding
{
    move |msg| {
        debug!("got message: {:?}", msg);

        let (name, value) = match msg {
            // only accept commands here
            Message::Command(ref name, ref value) => (name, value),
            _ => {
                debug!("not a command, dropping message");
                return Ok(())
            }
        };

        let val: C::Item = match binding.get_value(&name) {
            Some(v) => v.clone(),
            None => {
                debug!("could not find item for command");
                return Ok(())
            }
        };

        match val.set_value(value.clone()) {
            Ok(_) => {}
            Err(e) => warn!("error setting value from {:?}: {:?}", msg, e),
        };

        Ok(())
    }
}

fn binding_to_bus<B, V>(bus: B) -> impl FnMut(Notification<V>) -> Result<()>
    where V: Item + Sized,
          B: Bus
{
    move |notification| {
        let mut meta: Option<Meta> = None;
        let mut skip_state = false;
        let mut new_sub = false;
        let mut remove_sub = false;

        let val = match notification {
            Notification::Changed(v) => v,
            Notification::Added(v) => {
                meta = v.get_meta();
                skip_state = true;
                new_sub = true;
                v
            }
            Notification::Removed(v) => {
                remove_sub = true;
                skip_state = true;
                v
            }
        };

        if let Some(meta) = meta {
            if let Err(e) = bus.publish(Message::Meta(val.get_name(), meta)) {
                warn!("bus publish error: {:?}", e);
            }
        }

        if new_sub {
            if let Err(e) = bus.subscribe(&val.get_name(), SubType::Command) {
                warn!("bus subscribe error: {:?}", e);
            }
        }

        if remove_sub {
            if let Err(e) = bus.unsubscribe(&val.get_name(), SubType::Command) {
                warn!("bus unsubscribe error: {:?}", e);
            }
        }

        if skip_state {
            return Ok(());
        }

        let value = match val.get_value() {
            Ok(v) => v,
            Err(e) => {
                warn!("error getting item value: {:?}", e);
                return Ok(());
            }
        };

        if let Err(e) = bus.publish(Message::Update(val.get_name(), value)) {
            warn!("bus publish error: {:?}", e);
        }
        Ok(())
    }
}
