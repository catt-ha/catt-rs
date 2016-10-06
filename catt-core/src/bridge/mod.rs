use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;

use std::sync::mpsc::Receiver;

use binding::Notification;

pub mod config;
pub use self::config::Config;

use bus::Bus;
use bus::SubType;
use bus::Message;

use item::Item;
use item::Meta;

use std::thread::JoinHandle;
use std::error::Error as SError;

error_chain! {
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

pub struct Bridge<B, C> {
    #[allow(dead_code)]
    bus: Arc<Mutex<B>>,
    #[allow(dead_code)]
    binding: Arc<C>,
    handles: Vec<JoinHandle<()>>,
}

impl<B, C> Bridge<B, C>
    where B: ::bus::Bus + Send + 'static,
          C: ::binding::Binding
{
    pub fn new(cfg: Config<B::Config, C::Config>) -> Result<Self> {

        let (bus, messages) = match B::new(&cfg.bus) {
            Ok(b) => b,
            Err(e) => return Err(ErrorKind::Bus(Box::new(e)).into()),
        };
        let (binding, notifications) = match C::new(&cfg.binding) {
            Ok(b) => b,
            Err(e) => return Err(ErrorKind::Binding(Box::new(e)).into()),
        };

        let devices = binding.get_values();
        let bus = Arc::new(Mutex::new(bus));
        let binding = Arc::new(binding);

        let handles = vec![spawn_bus_to_binding(messages, devices.clone()),
                           spawn_binding_to_bus(notifications, bus.clone())];

        Ok(Bridge {
            bus: bus,
            binding: binding,
            handles: handles,
        })
    }

    pub fn join_all(self) {
        for h in self.handles {
            let _ = h.join();
        }
    }
}


fn spawn_bus_to_binding<V>(msgs: Receiver<Message>,
                           values: Arc<Mutex<BTreeMap<String, V>>>)
                           -> JoinHandle<()>
    where V: Send + 'static + Clone + Item
{
    ::std::thread::spawn(move || {
        for msg in msgs {
            debug!("got message: {:?}", msg);

            let (name, value) = match msg {
                // only accept commands here
                Message::Command(ref name, ref value) => (name, value),
                _ => {
                    debug!("not a command, dropping message");
                    continue;
                }
            };

            let val: V = match ::util::always_lock(values.lock()).get(name) {
                Some(v) => v.clone(),
                None => {
                    debug!("could not find item for command");
                    continue;
                }
            };

            match val.set_value(value.clone()) {
                Ok(_) => {}
                Err(e) => warn!("error setting value from {:?}: {:?}", msg, e),
            };
        }
    })
}

fn spawn_binding_to_bus<V, B>(notifications: Receiver<Notification<V>>,
                              bus: Arc<Mutex<B>>)
                              -> JoinHandle<()>
    where V: Send + 'static + Clone + Item,
          B: Send + 'static + Bus
{
    ::std::thread::spawn(move || {
        for notification in notifications {
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
                if let Err(e) = ::util::always_lock(bus.lock())
                    .publish(Message::Meta(val.get_name(), meta)) {
                    warn!("bus publish error: {:?}", e);
                }
            }

            if new_sub {
                if let Err(e) = ::util::always_lock(bus.lock())
                    .subscribe(&val.get_name(), SubType::Command) {
                    warn!("bus subscribe error: {:?}", e);
                }
            }

            if remove_sub {
                if let Err(e) = ::util::always_lock(bus.lock())
                    .unsubscribe(&val.get_name(), SubType::Command) {
                    warn!("bus unsubscribe error: {:?}", e);
                }
            }

            if skip_state {
                continue;
            }

            let value = match val.get_value() {
                Ok(v) => v,
                Err(e) => {
                    warn!("error getting item value: {:?}", e);
                    continue;
                }
            };

            if let Err(e) = ::util::always_lock(bus.lock())
                .publish(Message::Update(val.get_name(), value)) {
                warn!("bus publish error: {:?}", e);
            }
        }
    })
}
