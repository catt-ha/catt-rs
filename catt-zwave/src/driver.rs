use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use catt_core::util::always_lock;
use catt_core::binding::Binding;
use catt_core::binding::Notification;
use catt_core::value::Value;
use catt_core::item::Item as CItem;

use openzwave_stateful as ozw;
use openzwave_stateful::ValueID;
use openzwave_stateful::ValueGenre;
use openzwave_stateful::ValueType;
use openzwave_stateful::ZWaveNotification;

use config::Config;

use errors::*;
use item::Item;

use device::DB;

#[derive(Clone)]
pub struct ZWave {
    #[allow(dead_code)]
    ozw_manager: Arc<ozw::ZWaveManager>,
    // TODO improve this system - ideally, we should hide these behind another struct
    // so that only one call is needed to update both.
    items: Arc<Mutex<DB>>,
}

impl ZWave {
    pub fn new(cfg: &Config) -> Result<(ZWave, Receiver<Notification<Item>>)> {
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
            items: Arc::new(Mutex::new(Default::default())),
        };

        spawn_notification_thread(driver.clone(), cfg, tx, notifications);

        Ok((driver, rx))
    }

    pub fn get_manager(&self) -> Arc<ozw::ZWaveManager> {
        self.ozw_manager.clone()
    }
}

impl Binding for ZWave {
    type Config = Config;
    type Error = Error;
    type Item = Item;

    fn new(cfg: &Self::Config) -> Result<(Self, Receiver<Notification<Item>>)> {
        ZWave::new(cfg)
    }

    fn get_value(&self, name: &str) -> Option<Item> {
        always_lock(self.items.lock()).get_item(&String::from(name)).map(|i| i.clone())
    }
}

fn spawn_notification_thread(driver: ZWave,
                             cfg: Config,
                             output: Sender<Notification<Item>>,
                             rx: Receiver<ZWaveNotification>) {
    ::std::thread::spawn(move || {
        for zwave_notification in rx {
            let notification: Notification<Item> = match zwave_notification {
                ZWaveNotification::ControllerReady(c) => {
                    let home_id = c.get_home_id();
                    let controller = Item::controller(&format!("zwave_{}_Controller", home_id),
                                                      driver.clone(),
                                                      home_id);
                    always_lock(driver.items.lock())
                        .add_item(controller.get_name(), controller.clone());
                    output.send(Notification::Added(controller.clone())).unwrap();
                    Notification::Changed(controller)
                }
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
                    let mut db = always_lock(driver.items.lock());
                    let (name, exists) = match cfg.lookup_device(v) {
                        Some(name) => {
                            let exists = if let Some(_) = db.get_name(&v) {
                                warn!("duplicate match found for {}", name);
                                true
                            } else {
                                false
                            };
                            (name, exists)
                        }
                        None => {
                            if cfg.expose_unbound.unwrap_or(true) {
                                if let Some(name) = db.get_name(&v) {
                                    warn!("duplicate match found for unconfigured {}", name);
                                    (name.clone(), true)
                                } else {
                                    (format!("zwave_{}_{}_{}",
                                             v.get_home_id(),
                                             v.get_node_id(),
                                             v.get_label()),
                                     false)
                                }
                            } else {
                                debug!("no configured devices matched {}", v);
                                continue;
                            }
                        }
                    };
                    let item = if !exists {
                        debug!("adding value {} to db", name);
                        db.add_value(name.clone(), v)
                    } else {
                        Item::item(&name, v)
                    };
                    Notification::Added(item)
                }

                ZWaveNotification::ValueChanged(v) => {
                    if !should_expose(v) {
                        continue;
                    }
                    let db = always_lock(driver.items.lock());
                    let name = match db.get_name(&v) {
                        Some(n) => n,
                        None => continue,
                    };
                    let item = Item::item(&name, v);
                    debug!("value {} changed: {:?}", item.get_name(), item.get_value());
                    Notification::Changed(item)
                }

                ZWaveNotification::ValueRemoved(v) => {
                    if !should_expose(v) {
                        continue;
                    }
                    let mut db = always_lock(driver.items.lock());
                    let name = match db.get_name(&v) {
                        Some(n) => n.clone(),
                        None => continue,
                    };
                    debug!("removing value {} from db", name);
                    Notification::Removed(match db.remove_value(v) {
                        Some(it) => it,
                        None => Item::item(&name, v),
                    })
                }

                ZWaveNotification::Generic(s) => {
                    if s.contains("Type_DriverRemoved") {
                        warn!("controller removed! shutting down.");
                        ::std::process::exit(1);
                    }
                    continue;
                }

                ZWaveNotification::StateStarting(c) => {
                    let db_name = format!("zwave_{}_Controller", c.get_home_id());
                    let db = always_lock(driver.items.lock());
                    match db.get_item(&db_name) {
                        Some(controller) => Notification::Changed(controller.clone()),
                        None => {
                            debug!("controller not found in item db");
                            continue;
                        }
                    }
                }
                ZWaveNotification::StateCompleted(c) => {
                    let db_name = format!("zwave_{}_Controller", c.get_home_id());
                    let db = always_lock(driver.items.lock());
                    match db.get_item(&db_name) {
                        Some(controller) => {
                            let _ = controller.set_value(Value::String("idle".into()));
                            Notification::Changed(controller.clone())
                        }
                        None => {
                            debug!("controller not found in item db");
                            continue;
                        }
                    }
                }

                ZWaveNotification::StateFailed(c) => {
                    let db_name = format!("zwave_{}_Controller", c.get_home_id());
                    let db = always_lock(driver.items.lock());
                    match db.get_item(&db_name) {
                        Some(controller) => {
                            let _ = controller.set_value(Value::String("failed".into()));
                            Notification::Changed(controller.clone())
                        }
                        None => {
                            debug!("controller not found in item db");
                            continue;
                        }
                    }
                }

                _ => {
                    debug!("unmatched notification: {}", zwave_notification);
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
