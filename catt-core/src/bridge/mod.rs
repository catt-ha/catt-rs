use tokio_core::reactor::Handle;

use futures;
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

pub fn new<B, C>(handle: &Handle,
                 cfg: Config<B::Config, C::Config>)
                 -> Result<impl Future<Item = (), Error = Error>>
    where B: Bus + 'static,
          C: Binding
{

    let (bus, messages) = match B::new(handle, &cfg.bus.unwrap_or_default()) {
        Ok(b) => b,
        Err(e) => return Err(ErrorKind::Bus(Box::new(e)).into()),
    };
    let (binding, notifications) = match C::new(handle, &cfg.binding.unwrap_or_default()) {
        Ok(b) => b,
        Err(e) => return Err(ErrorKind::Binding(Box::new(e)).into()),
    };

    let msg_fut = bus_to_binding(handle, messages.map_err(Error::from), binding);
    let not_fut = binding_to_bus(handle, notifications.map_err(Error::from), bus);

    Ok(msg_fut.select(not_fut).map(|_| ()).map_err(|(e, _)| e))
}

pub fn from_file<B, C>(handle: &Handle,
                       config_file: &str)
                       -> Result<impl Future<Item = (), Error = Error>>
    where B: Bus + 'static,
          C: Binding
{
    let cfg: Config<B::Config, C::Config> = Config::from_file(config_file)?;
    Ok(new::<B, C>(handle, cfg)?)
}

fn bus_to_binding<B, S>(handle: &Handle,
                        msg_stream: S,
                        binding: B)
                        -> impl Future<Item = (), Error = Error>
    where B: Binding,
          S: Stream<Item = Message, Error = Error>
{
    let handle = handle.clone();
    msg_stream.filter_map(|msg| {
            debug!("got message: {:?}", msg);
            match msg {
                // only accept commands here
                Message::Command(name, value) => Some((name, value)),
                _ => {
                    debug!("not a command, dropping message");
                    None
                }
            }
        })
        .for_each(move |(name, value)| {
            handle.spawn(binding.get_value(&name)
                .map_err(|e| {
                    warn!("error getting item from binding: {:#?}", e);
                    e
                })
                .and_then(|opt| {
                    match opt {
                        Some(it) => it.set_value(value),
                        None => futures::finished(()).boxed(),
                    }
                })
                .map_err(|e| {
                    warn!("error setting item value: {:#?}", e);
                }));
            Ok(())
        })
}

fn binding_to_bus<B, S, V>(handle: &Handle,
                           notification_stream: S,
                           bus: B)
                           -> impl Future<Item = (), Error = Error>
    where V: Item + Sized + 'static,
          B: Bus + 'static,
          S: Stream<Item = Notification<V>, Error = Error>
{
    let handle = handle.clone();

    // need this so that we can use bus inside of a FnOnce
    let bus = ::std::rc::Rc::new(bus);

    notification_stream.for_each(move |notification| {
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

        let name = val.get_name();

        if let Some(meta) = meta {
            handle.spawn(bus.publish(Message::Meta(name.clone(), meta))
                .map_err(|e| warn!("error publishing metadata: {:#?}", e)))
        }

        if new_sub {
            handle.spawn(bus.subscribe(&name, SubType::Command)
                .map_err(|e| warn!("bus subscribe error: {:?}", e)));
        }

        if remove_sub {
            handle.spawn(bus.unsubscribe(&name, SubType::Command)
                .map_err(|e| warn!("bus unsubscribe error: {:?}", e)));
        }

        if !skip_state {
            let state = val.get_value()
                .map_err(|e| warn!("error getting item value: {:#?}", e))
                .map(|v| Message::Update(name, v));
            let bus = bus.clone();
            let publish = state.and_then(move |msg| {
                bus.publish(msg)
                    .map_err(|e| warn!("new state publish error: {:#?}", e))
            });

            handle.spawn(publish);
        }

        Ok(())
    })
}
