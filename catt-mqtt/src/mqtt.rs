use rumqtt;

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
    client: Option<rumqtt::MqttClient>,
    requester: Option<rumqtt::MqRequest>,
}

impl Mqtt {
    pub fn with_config(cfg: &Config) -> Result<Mqtt> {
        let mut client_options = rumqtt::MqttOptions::new()
            .set_keep_alive(5)
            .set_reconnect(3);

        // let mut net_opts = netopt::NetworkOptions::new();
        // if cfg.tls.unwrap_or(false) {
        //     net_opts.tls(Default::default());
        // }

        match &cfg.client_id {
            &Some(ref id) => client_options = client_options.set_client_id(&id),
            &None => {}
        };

        let addr: &str = cfg.broker.as_ref();
        client_options = client_options.broker(addr);


        let client = rumqtt::MqttClient::new(client_options);

        Ok(Mqtt {
            cfg: cfg.clone(),
            client: Some(client),
            requester: None,
        })
    }

    pub fn with_callback<F>(mut self, cb: F) -> Self
        where F: Fn(rumqtt::Message) + Send + Sync + 'static
    {
        self.client = match self.client.take() {
            Some(cl) => Some(cl.message_callback(cb)),
            None => None,
        };

        self
    }

    pub fn start(mut self) -> Result<Self> {
        let (client, requester) = match (self.client.take(), self.requester.take()) {
            (Some(cl), _) => {
                let requester = cl.start()?;
                (None, Some(requester))
            }
            (None, Some(req)) => (None, Some(req)),
            (cl, req) => (cl, req),
        };

        self.client = client;
        self.requester = requester;
        Ok(self)
    }

    pub fn publish(&self, path: &str, state: &[u8]) -> Result<()> {
        let pub_path = match self.cfg.item_base {
            Some(ref b) => format!("{}/{}", b, path),
            None => format!("{}/{}", MQTT_BASE_DEFAULT, path),
        };

        match self.requester {
            Some(ref req) => Ok(req.publish(&pub_path, rumqtt::QoS::Level0, state.into())?),
            None => Err(ErrorKind::NotStarted.into()),
        }
    }

    pub fn subscribe(&self, path: &str) -> Result<()> {
        let sub_path = match self.cfg.item_base {
            Some(ref b) => format!("{}/{}", b, path),
            None => format!("{}/{}", MQTT_BASE_DEFAULT, path),
        };

        match self.requester {
            Some(ref req) => Ok(req.subscribe(vec![(&sub_path, rumqtt::QoS::Level0)])?),
            None => Err(ErrorKind::NotStarted.into()),
        }
    }
}

pub struct MqttBus {
    client: Mqtt,
    messages: Arc<Mutex<Receiver<Message>>>,
}

impl MqttBus {
    pub fn with_config(cfg: &Config) -> Result<Self> {
        let (tx, rx) = channel();
        let tx = Mutex::new(tx);

        let client = Mqtt::with_config(cfg)?
            .with_callback(move |message| {
                let topic = message.topic.as_str().split("/").collect::<Vec<&str>>();

                if topic.len() < 2 {
                    warn!("message with invalid path received: {}",
                          message.topic.as_str());
                    return;
                }

                let item_name = topic[topic.len() - 2];

                let message_type_str = topic[topic.len() - 1];
                let message_type = match message_type_str {
                    "state" => MessageType::Update,
                    "command" => MessageType::Command,
                    _ => {
                        warn!("invalid message type: {}", message_type_str);
                        return;
                    }
                };

                let value: Vec<u8> = (&*message.payload).clone();

                let message = Message {
                    message_type: message_type,
                    item_name: String::from(item_name),
                    value: value,
                };

                match ::catt_core::util::always_lock(tx.lock()).send(message) {
                    Ok(_) => {}
                    Err(e) => warn!("channel send error: {}", e),
                }
            })
            .start()?;

        Ok(MqttBus {
            client: client,
            messages: Arc::new(Mutex::new(rx)),
        })
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
