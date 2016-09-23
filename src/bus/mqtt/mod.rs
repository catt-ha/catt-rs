pub mod errors;

use netopt;

use mqttc;
use mqttc::PubSub;
use mqttc::PubOpt;
use mqttc::ToSubTopics;

use mqtt3::QoS;
use mqtt3::SubscribeTopic;
use mqtt3::Message;

use config::BrokerConfig;
use config::DeviceConfig;
use config::MQTT_BASE_DEFAULT;
use config::MQTT_QOS_DEFAULT;

use std::sync::Arc;
use std::sync::MutexGuard;

use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;

use manager::device::DeviceDB;

pub use self::errors::*;

pub struct Mqtt {
    cfg: BrokerConfig,
    client: Arc<Mutex<mqttc::Client>>,
    subs: Option<Vec<String>>,
}

impl Mqtt {
    pub fn with_config(cfg: &BrokerConfig) -> Result<Mqtt> {
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
            subs: None,
        })
    }

    pub fn publish(&self, name: &str, state: &str) -> Result<()> {
        let pub_path = match self.cfg.item_base {
            Some(ref b) => format!("{}/{}/state", b, name),
            None => format!("{}/{}/state", MQTT_BASE_DEFAULT, name),
        };

        let pub_opt = PubOpt::new(QoS::from_u8(self.cfg.qos.clone().unwrap_or(MQTT_QOS_DEFAULT))?,
                                  false);

        Ok(self.get_client().publish(pub_path, state, pub_opt)?)
    }

    pub fn subscribe(&mut self, devices: &DeviceDB) -> Result<()> {
        if self.subs.is_some() {
            let unsubs = self.subs.take().unwrap();
            self.get_client().unsubscribe(unsubs)?;
        }

        let subs: Vec<String> = devices.by_name
            .keys()
            .cloned()
            .map(|s| {
                format!("{}/{}/command",
                        self.cfg.item_base.clone().unwrap_or(MQTT_BASE_DEFAULT.into()),
                        s)
            })
            .collect();

        let mut subtopics: Vec<SubscribeTopic> = Vec::new();
        for s in subs.iter() {
            debug!("subscribing to {}", s);
            let mut topics = s.as_str()
                .to_subscribe_topics()?
                .collect();
            subtopics.append(&mut topics);
        }

        self.get_client().subscribe(subtopics)?;
        self.subs = Some(subs);
        Ok(())
    }

    fn get_client(&self) -> MutexGuard<mqttc::Client> {
        match self.client.lock() {
            Ok(cl) => cl,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    pub fn get_message(&self) -> Result<Option<Box<Message>>> {
        Ok(self.get_client().await()?)
    }
}
