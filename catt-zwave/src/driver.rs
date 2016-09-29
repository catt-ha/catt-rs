use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use catt_core::util::CVar;
use catt_core::util::always_lock;
use catt_core::binding::Binding;
use catt_core::binding::Notification;

use openzwave_stateful as ozw;
use openzwave_stateful::ValueID;
use openzwave_stateful::ZWaveNotification;

use config::ZWaveConfig;
use super::errors::*;
use super::value::Value;

pub struct ZWave {
    #[allow(dead_code)]
    ozw_manager: Arc<ozw::ZWaveManager>,
    notifications: Arc<Mutex<Receiver<Notification<Value>>>>,
    values: Arc<Mutex<BTreeMap<ValueID, (String, u8)>>>,
}

impl ZWave {
    pub fn new(cfg: &ZWaveConfig) -> Result<ZWave> {
        let cfg = cfg.clone();
        let (manager, notifications) = {
            let sys_config: ozw::ConfigPath = cfg.sys_config
                .as_ref()
                .map(|c| ozw::ConfigPath::Custom(&c))
                .unwrap_or(ozw::ConfigPath::Default);

            let user_config: &str =
                cfg.user_config.as_ref().map(|c| c.as_ref()).unwrap_or("./config");

            ozw::init(&ozw::InitOptions {
                devices: cfg.port.clone().map(|x| vec![x.into()]),
                config_path: sys_config,
                user_path: user_config,
            })?
        };

        let value_db = Arc::new(Mutex::new(BTreeMap::new()));
        let ready = CVar::new();

        let (tx, rx) = channel();

        spawn_notification_thread(value_db.clone(), cfg, tx, notifications, ready.clone());

        let _ = ready.wait();

        Ok(ZWave {
            ozw_manager: manager,
            notifications: Arc::new(Mutex::new(rx)),
            values: value_db,
        })
    }
}

impl Binding for ZWave {
    type Error = Error;
    type Item = Value;

    fn get_values(&self) -> BTreeMap<String, Self::Item> {
        let values_lock = ::catt_core::util::always_lock(self.values.lock());

        values_lock.iter()
            .map(|(k, v)| (v.0.clone(), Value::new(&v.0, *k)))
            .collect()
    }

    fn notifications(&self) -> Arc<Mutex<Receiver<Notification<Self::Item>>>> {
        self.notifications.clone()
    }
}

fn spawn_notification_thread(db: Arc<Mutex<BTreeMap<ValueID, (String, u8)>>>,
                             cfg: ZWaveConfig,
                             output: Sender<Notification<Value>>,
                             rx: Receiver<ZWaveNotification>,
                             ready: CVar) {
    ::std::thread::spawn(move || {
        for zwave_notification in rx {
            let notification: Notification<Value> = match zwave_notification {
                ZWaveNotification::AllNodesQueried(_) |
                ZWaveNotification::AwakeNodesQueried(_) |
                ZWaveNotification::AllNodesQueriedSomeDead(_) => {
                    debug!("Controller ready");
                    ready.notify_all();
                    continue;
                }

                ZWaveNotification::ValueAdded(v) => {
                    let name = match cfg.lookup_device(v) {
                        Some((name, strength)) => {
                            let mut db = always_lock(db.lock());
                            let better = if let Some(&(_, db_strength)) = db.get(&v) {
                                strength > db_strength
                            } else {
                                true
                            };
                            if better {
                                db.insert(v, (name.clone(), strength));
                            }
                            name
                        }
                        None => {
                            debug!("no configured devices matched {}", v);
                            continue;
                        }
                    };
                    Notification::Added(Value::new(&name, v))
                }

                ZWaveNotification::ValueChanged(v) => {
                    let db = always_lock(db.lock());
                    let name = match db.get(&v) {
                        Some(&(ref n, _)) => n,
                        None => continue,
                    };
                    debug!("value {} changed: {}", name, v);
                    Notification::Changed(Value::new(&name, v))
                }

                ZWaveNotification::ValueRemoved(v) => {
                    let mut db = always_lock(db.lock());
                    let name = match db.get(&v) {
                        Some(&(ref n, _)) => n.clone(),
                        None => continue,
                    };
                    debug!("removing value {} from db", name);
                    db.remove(&v);
                    Notification::Removed(Value::new(&name, v))
                }

                _ => {
                    // debug!("unmatched notification: {}", n);
                    continue;
                }
            };

            match output.send(notification) {
                Ok(_) => {}
                Err(e) => {
                    warn!("zwave notification send error: {}", e);
                    return;
                }
            }
        }
    });
}
