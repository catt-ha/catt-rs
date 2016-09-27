use netopt;

use mqttc;
use mqttc::PubSub;
use mqttc::PubOpt;

use mqtt3::QoS;

use config::Config;
use config::MQTT_BASE_DEFAULT;
use config::MQTT_QOS_DEFAULT;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use std::thread;

use catt_core::bus::Bus;
use catt_core::bus::Message;
use catt_core::bus::MessageType;
use catt_core::bus::SubType;

use errors::*;

pub struct Mqtt {
    cfg: Config,
    client: Arc<Mutex<mqttc::Client>>,
}

impl Mqtt {
    pub fn with_config(cfg: &Config) -> Result<Mqtt> {
        let mut net_opts = netopt::NetworkOptions::new();
        if cfg.tls.unwrap_or(false) {
            net_opts.tls(Default::default());
        }

        let mut client_opts = mqttc::ClientOptions::new();

        cfg.client_id.clone().map(|id| client_opts.set_client_id(id.into()));

        let addr: &str = cfg.broker.as_ref();
        let client = Arc::new(Mutex::new(client_opts.connect(addr, net_opts)?));

        Ok(Mqtt {
            cfg: cfg.clone(),
            client: client,
        })
    }

    pub fn publish(&self, path: &str, state: &[u8]) -> Result<()> {
        let pub_path = match self.cfg.item_base {
            Some(ref b) => format!("{}/{}", b, path),
            None => format!("{}/{}", MQTT_BASE_DEFAULT, path),
        };

        let pub_opt = PubOpt::new(QoS::from_u8(self.cfg.qos.clone().unwrap_or(MQTT_QOS_DEFAULT))?,
                                  false);

        debug!("publishing to {}", pub_path);
        Ok(self.get_client().publish(pub_path, Vec::from(state), pub_opt)?)
    }

    fn get_client(&self) -> MutexGuard<mqttc::Client> {
        ::catt_core::util::always_lock(self.client.lock())
    }

    pub fn get_message(&self) -> Result<Option<(String, Arc<Vec<u8>>)>> {
        let message = self.get_client().await()?;

        Ok(message.map(|m| {
            let value = m.payload.clone();
            (m.topic.path, value)
        }))
    }

    pub fn subscribe(&self, path: &str) -> Result<()> {
        let sub_path = match self.cfg.item_base {
            Some(ref b) => format!("{}/{}", b, path),
            None => format!("{}/{}", MQTT_BASE_DEFAULT, path),
        };

        debug!("subscribing to {}", sub_path);

        Ok(self.get_client().subscribe(sub_path.as_str())?)
    }
}

fn spawn_message_thread(client: Arc<Mutex<mqttc::Client>>, tx: Sender<Message>) {
    thread::spawn(move || {
        loop {
            debug!("locking client");
            let message = {
                let mut cl = ::catt_core::util::always_lock(client.lock());

                debug!("receiving message");
                cl.await()
            };

            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    warn!("mqtt await error: {:?}", e);
                    continue;
                }
            };


            let message = match message {
                Some(m) => m,
                None => continue,
            };

            let topic = message.topic.path.split("/").collect::<Vec<&str>>();

            if topic.len() < 2 {
                warn!("message with invalid path received: {}", message.topic.path);
                continue;
            }

            let item_name = topic[topic.len() - 2];

            let message_type_str = topic[topic.len() - 1];
            let message_type = match message_type_str {
                "state" => MessageType::Update,
                "command" => MessageType::Command,
                _ => {
                    warn!("invalid message type: {}", message_type_str);
                    continue;
                }
            };

            let value: Vec<u8> = (&*message.payload).clone();

            let message = Message {
                message_type: message_type,
                item_name: String::from(item_name),
                value: value,
            };

            match tx.send(message) {
                Ok(_) => {}
                Err(e) => warn!("channel send error: {}", e),
            }
        }
    });
}

pub struct MqttBus {
    client: Mqtt,
    messages: Arc<Mutex<Receiver<Message>>>,
}

impl From<Mqtt> for MqttBus {
    fn from(other: Mqtt) -> MqttBus {
        let (tx, rx) = channel();
        spawn_message_thread(other.client.clone(), tx);
        MqttBus {
            client: other,
            messages: Arc::new(Mutex::new(rx)),
        }
    }
}

impl Bus for MqttBus {
    type Error = Error;

    fn publish(&self, message: Message) -> Result<()> {
        let item_name = &message.item_name;
        let value = &message.value;
        let message_type = match &message.message_type {
            &MessageType::Update => "state",
            &MessageType::Command => "command",
        };
        let path = format!("{}/{}", item_name, message_type);
        self.client.publish(&path, value)
    }

    fn subscribe(&self, item_name: &str, sub_type: SubType) -> Result<()> {
        match sub_type {
            SubType::Update => self.client.subscribe(&format!("{}/state", item_name)),
            SubType::Command => self.client.subscribe(&format!("{}/command", item_name)),
            SubType::All => self.client.subscribe(&format!("{}/#", item_name)),
        }
    }

    fn messages(&self) -> Arc<Mutex<Receiver<Message>>> {
        self.messages.clone()
    }
}


#[cfg(test)]
mod test {
    use env_logger::LogBuilder;

    use catt_core::bus::Bus;
    use catt_core::bus::SubType;
    use catt_core::bus::Message;
    use catt_core::bus::MessageType;

    use super::*;
    use super::super::config::*;

    fn new_client() -> Mqtt {
        Mqtt::with_config(&Config {
                broker: "10.8.0.1:1883".into(),
                item_base: Some("catt/items".into()),
                client_id: Some("tethys_test".into()),
                ..Default::default()
            })
            .unwrap()
    }

    fn new_bus() -> MqttBus {
        new_client().into()
    }

    #[test]
    fn stuff() {
        let _ = LogBuilder::new().parse("debug").init();
        let client = new_bus();
        let messages = client.messages();
        client.subscribe("test_item", SubType::Command).unwrap();
        client.publish(Message {
                item_name: "test_item".into(),
                message_type: MessageType::Command,
                value: "Testing!".into(),
            })
            .unwrap();
        let msg = messages.lock().unwrap().recv().unwrap();
        info!("got: {:?}", String::from_utf8(msg.value).unwrap());
    }
}
