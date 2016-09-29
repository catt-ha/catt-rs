use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;

use std::sync::mpsc::Receiver;

use binding::Notification;

use bus::Bus;
use bus::SubType;
use bus::Message;
use bus::MessageType;

use item::Item;

pub struct Bridge<B, C> {
    #[allow(dead_code)]
    bus: Arc<Mutex<B>>,
    #[allow(dead_code)]
    binding: Arc<C>,
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
                Err(e) => warn!("subscribe error: {}", e),
                _ => {}
            }
        }

        let bus_messages = bus.messages();
        let bus = Arc::new(Mutex::new(bus));
        let binding = Arc::new(binding);

        let devices = Arc::new(Mutex::new(values));

        spawn_bus_to_binding(bus_messages, devices.clone());
        spawn_binding_to_bus(binding.notifications(), bus.clone());

        Bridge {
            bus: bus,
            binding: binding,
        }
    }
}


fn spawn_bus_to_binding<V>(msgs: Arc<Mutex<Receiver<Message>>>,
                           values: Arc<Mutex<BTreeMap<String, V>>>)
    where V: Send + 'static + Clone + Item
{
    ::std::thread::spawn(move || {
        loop {
            let msg = match ::util::always_lock(msgs.lock()).recv() {
                Ok(m) => m,
                Err(_) => break,
            };

            match msg.message_type {
                // only accept commands here
                MessageType::Update => continue,
                _ => {}
            }

            let val: V = match ::util::always_lock(values.lock()).get(&msg.item_name) {
                Some(v) => v.clone(),
                None => continue,
            };

            match val.set_value(msg.value) {
                Ok(_) => {}
                Err(e) => warn!("error setting value from bus: {}", e),
            };
        }
    });
}
fn spawn_binding_to_bus<V, B>(notifications: Arc<Mutex<Receiver<Notification<V>>>>,
                              bus: Arc<Mutex<B>>)
    where V: Send + 'static + Clone + Item,
          B: Send + 'static + Bus
{
    ::std::thread::spawn(move || {
        loop {
            let notification = match ::util::always_lock(notifications.lock()).recv() {
                Ok(n) => n,
                Err(_) => break,
            };

            let val = match notification {
                Notification::Added(v) |
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

            match ::util::always_lock(bus.lock()).publish(Message {
                message_type: MessageType::Update,
                item_name: val.get_name(),
                value: value,
            }) {
                Ok(_) => {}
                Err(e) => warn!("bus publish error: {:?}", e),
            };
        }
    });
}
