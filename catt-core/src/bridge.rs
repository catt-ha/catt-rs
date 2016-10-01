use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;

use std::sync::mpsc::Receiver;

use binding::Notification;

use bus::Bus;
use bus::SubType;
use bus::Message;

use item::Item;
use item::Meta;

use std::thread::JoinHandle;

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
    pub fn new(bus: B, binding: C) -> Self {
        let values = binding.get_values();
        for (name, _) in values.iter() {
            let res = bus.subscribe(name, SubType::Command);
            match res {
                Err(e) => warn!("subscribe error: {:?}", e),
                _ => {}
            }
        }

        let bus_messages = bus.messages();
        let bus = Arc::new(Mutex::new(bus));
        let binding = Arc::new(binding);

        let devices = Arc::new(Mutex::new(values));

        let handles = vec![spawn_bus_to_binding(bus_messages, devices.clone()),
                           spawn_binding_to_bus(binding.notifications(), bus.clone())];

        Bridge {
            bus: bus,
            binding: binding,
            handles: handles,
        }
    }

    pub fn join_all(self) {
        for h in self.handles {
            let _ = h.join();
        }
    }
}


fn spawn_bus_to_binding<V>(msgs: Arc<Mutex<Receiver<Message>>>,
                           values: Arc<Mutex<BTreeMap<String, V>>>)
                           -> JoinHandle<()>
    where V: Send + 'static + Clone + Item
{
    ::std::thread::spawn(move || {
        loop {
            let msg = match ::util::always_lock(msgs.lock()).recv() {
                Ok(m) => m,
                Err(_) => break,
            };

            let (name, value) = match msg {
                // only accept commands here
                Message::Command(name, value) => (name, value),
                _ => continue,
            };

            let val: V = match ::util::always_lock(values.lock()).get(&name) {
                Some(v) => v.clone(),
                None => continue,
            };

            match val.set_value(value) {
                Ok(_) => {}
                Err(e) => warn!("error setting value from bus: {:?}", e),
            };
        }
    })
}

fn spawn_binding_to_bus<V, B>(notifications: Arc<Mutex<Receiver<Notification<V>>>>,
                              bus: Arc<Mutex<B>>)
                              -> JoinHandle<()>
    where V: Send + 'static + Clone + Item,
          B: Send + 'static + Bus
{
    ::std::thread::spawn(move || {
        loop {
            let notification = match ::util::always_lock(notifications.lock()).recv() {
                Ok(n) => n,
                Err(_) => break,
            };

            let mut meta: Option<Meta> = None;
            let mut skip_state = false;

            let val = match notification {
                Notification::Added(v) => {
                    meta = v.get_meta();
                    skip_state = true;
                    v
                }
                Notification::Changed(v) => v,
                _ => continue,
            };

            let value = match val.get_value() {
                Ok(v) => v,
                Err(e) => {
                    warn!("error getting item value: {:?}", e);
                    continue;
                }
            };

            if let Some(meta) = meta {
                if let Err(e) = ::util::always_lock(bus.lock())
                    .publish(Message::Meta(val.get_name(), meta)) {
                    warn!("bus publish error: {:?}", e);
                }
            }

            if skip_state {
                continue;
            }

            if let Err(e) = ::util::always_lock(bus.lock())
                .publish(Message::Update(val.get_name(), value)) {
                warn!("bus publish error: {:?}", e);
            }
        }
    })
}
