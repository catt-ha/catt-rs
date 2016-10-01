use std::collections::BTreeMap;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use catt_core::util::always_lock;
use catt_core::binding::Binding;
use catt_core::binding::Notification;

use openzwave_stateful as ozw;
use openzwave_stateful::ValueID;
use openzwave_stateful::ValueGenre;
use openzwave_stateful::ValueType;
use openzwave_stateful::ZWaveNotification;

use config::ZWaveConfig;
use super::errors::*;
use super::item::Item;

#[derive(Clone)]
pub struct ZWave {
    #[allow(dead_code)]
    ozw_manager: Arc<ozw::ZWaveManager>,
    notifications: Arc<Mutex<Receiver<Notification<Item>>>>,
    values: Arc<Mutex<BTreeMap<ValueID, String>>>,
    catt_values: Arc<Mutex<BTreeMap<String, Item>>>,
}

impl ZWave {
    pub fn new(cfg: &ZWaveConfig) -> Result<ZWave> {
        let cfg = cfg.clone();
        let (manager, notifications) = {
            let sys_config: ozw::ConfigPath = cfg.sys_config
                .as_ref()
                .map(|c| ozw::ConfigPath::Custom(&c))
                .unwrap_or(ozw::ConfigPath::Custom("/etc/openzwave"));

            let user_config: &str =
                cfg.user_config.as_ref().map(|c| c.as_ref()).unwrap_or("./config");

            ozw::init(&ozw::InitOptions {
                devices: cfg.port.clone().map(|x| vec![x.into()]),
                config_path: sys_config,
                user_path: user_config,
            })?
        };

        let (tx, rx) = channel();

        let driver = ZWave {
            ozw_manager: manager,
            notifications: Arc::new(Mutex::new(rx)),
            values: Default::default(),
            catt_values: Default::default(),
        };

        spawn_notification_thread(driver.clone(), cfg, tx, notifications);

        Ok(driver)
    }
}

impl Binding for ZWave {
    type Error = Error;
    type Item = Item;

    fn get_values(&self) -> Arc<Mutex<BTreeMap<String, Self::Item>>> {
        self.catt_values.clone()
    }

    fn notifications(&self) -> Arc<Mutex<Receiver<Notification<Self::Item>>>> {
        self.notifications.clone()
    }
}

fn spawn_notification_thread(driver: ZWave,
                             cfg: ZWaveConfig,
                             output: Sender<Notification<Item>>,
                             rx: Receiver<ZWaveNotification>) {
    ::std::thread::spawn(move || {
        for zwave_notification in rx {
            let notification: Notification<Item> = match zwave_notification {
                ZWaveNotification::AllNodesQueried(_) |
                ZWaveNotification::AwakeNodesQueried(_) |
                ZWaveNotification::AllNodesQueriedSomeDead(_) => {
                    debug!("Controller ready");
                    driver.ozw_manager.write_configs();
                    continue;
                }

                ZWaveNotification::ValueAdded(v) => {
                    if !should_expose(v) {
                        continue;
                    }
                    let mut db = always_lock(driver.values.lock());
                    let (name, exists) = match cfg.lookup_device(v) {
                        Some(name) => {
                            let exists = if let Some(_) = db.get(&v) {
                                warn!("duplicate match found for {}", name);
                                true
                            } else {
                                false
                            };
                            (name, exists)
                        }
                        None => {
                            if cfg.expose_unbound {
                                if let Some(name) = db.get(&v) {
                                    warn!("duplicate match found for unconfigured {}", name);
                                    (name.clone(), true)
                                } else {
                                    (format!("zwave_{}_{}", v.get_node_id(), v.get_label()), false)
                                }
                            } else {
                                debug!("no configured devices matched {}", v);
                                continue;
                            }
                        }
                    };
                    if !exists {
                        db.insert(v, name.clone());
                        always_lock(driver.catt_values.lock())
                            .insert(name.clone(), Item::new(&name, v));
                    }
                    Notification::Added(Item::new(&name, v))
                }

                ZWaveNotification::ValueChanged(v) => {
                    if !should_expose(v) {
                        continue;
                    }
                    let db = always_lock(driver.values.lock());
                    let name = match db.get(&v) {
                        Some(n) => n,
                        None => continue,
                    };
                    debug!("value {} changed: {}", name, v);
                    Notification::Changed(Item::new(&name, v))
                }

                ZWaveNotification::ValueRemoved(v) => {
                    if !should_expose(v) {
                        continue;
                    }
                    let mut db = always_lock(driver.values.lock());
                    let name = match db.get(&v) {
                        Some(n) => n.clone(),
                        None => continue,
                    };
                    debug!("removing value {} from db", name);
                    db.remove(&v);
                    always_lock(driver.catt_values.lock()).remove(&name);
                    Notification::Removed(Item::new(&name, v))
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

fn should_expose(v: ValueID) -> bool {
    match v.get_genre() {
        ValueGenre::ValueGenre_Basic |
        ValueGenre::ValueGenre_User => {}
        _ => return false,
    }

    match v.get_type() {
        ValueType::ValueType_Bool |
        ValueType::ValueType_Byte |
        ValueType::ValueType_Decimal |
        ValueType::ValueType_Int |
        ValueType::ValueType_Short |
        ValueType::ValueType_String |
        ValueType::ValueType_Raw => {}
        _ => return false,
    }
    true
}
