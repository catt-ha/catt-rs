use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use std::fs;

use catt_core::util::always_lock;
use catt_core::binding::Binding;
use catt_core::binding::Notification;
use catt_core::value::Value;
use catt_core::item::Item as CItem;

use openzwave as ozw;
use openzwave::manager::Manager;
use openzwave::options::Options;
use openzwave::value_classes::value_id::ValueID;
use openzwave::value_classes::value_id::ValueGenre;
use openzwave::value_classes::value_id::ValueType;
use openzwave::notification::Notification as ZWaveNotification;

use openzwave::notification::NotificationType;
use openzwave::notification::ControllerState;

use serial_ports::{ListPortInfo, ListPorts};
use serial_ports::ListPortType::UsbPort;

use futures::{self, Future, BoxFuture};

use tokio_core::reactor::Handle;
use tokio_core::channel::channel;
use tokio_core::channel::Sender;
use tokio_core::channel::Receiver;

use config::Config;

use errors::*;
use item::Item;

use device::DB;

#[cfg(windows)]
fn get_default_devices() -> Vec<String> {
    vec!["\\\\.\\COM6".to_owned()]
}

#[cfg(unix)]
fn is_usb_zwave_device(port: &ListPortInfo) -> bool {
    let default_usb_devices = [// VID     PID
                               // -----   -----
                               (0x0658, 0x0200), // Aeotech Z-Stick Gen-5
                               (0x0658, 0x0280), // UZB1
                               (0x10c4, 0xea60) /* Aeotech Z-Stick S2 */];

    // Is it one of the vid/pids in the table?
    if let UsbPort(ref info) = port.port_type {
        default_usb_devices.contains(&(info.vid, info.pid))
    } else {
        false
    }
}

#[cfg(unix)]
fn get_default_devices() -> Vec<String> {

    // Enumerate all of the serial devices and see if any of them match our
    // known VID:PID.

    let mut ports: Vec<String> = Vec::new();
    let usb_ports: Vec<String> = ListPorts::new()
        .iter()
        .filter(|port| is_usb_zwave_device(port))
        .map(|port| port.device.to_string_lossy().into_owned())
        .collect();
    ports.extend(usb_ports);
    if ports.is_empty() {
        // The following is only included temporarily until we can get a more
        // comprehensive list of VIDs and PIDs.

        error!("[OpenzwaveStateful] Unable to locate ZWave USB dongle. The following VID:PIDs \
                were found:");
        for port in ListPorts::new().iter() {
            if let UsbPort(ref info) = port.port_type {
                error!("[OpenzwaveStateful]   {:04x}:{:04x} {}",
                       info.vid,
                       info.pid,
                       port.device.display());
            }
        }

        // The following should be removed, once we have all of the devices captured using the above

        let default_devices = ["/dev/cu.usbserial", // MacOS X (presumably)
                               "/dev/cu.SLAB_USBtoUART", // MacOS X (Aeotech Z-Stick S2)
                               "/dev/cu.usbmodem14211", // Yoric (Aeotech Z-Stick Gen-5)
                               "/dev/cu.usbmodem1421", // Isabel (UZB Static Controller)
                               "/dev/ttyUSB0", // Linux (Aeotech Z-Stick S2)
                               "/dev/ttyACM0" /* Linux (Aeotech Z-Stick Gen-5) */];

        if let Some(default_device) =
            default_devices.iter()
                .find(|device_name| fs::metadata(device_name).is_ok())
                .map(|&str| str.to_owned()) {
            ports.push(default_device);
        }
    }
    ports
}

#[derive(Clone)]
pub struct ZWave {
    #[allow(dead_code)]
    ozw_manager: Arc<Mutex<ozw::manager::Manager>>,
    // TODO improve this system - ideally, we should hide these behind another struct
    // so that only one call is needed to update both.
    items: Arc<Mutex<DB>>,
}

impl ZWave {
    pub fn new(handle: &Handle, cfg: &Config) -> Result<(ZWave, Receiver<Notification<Item>>)> {
        let cfg = cfg.clone();

        let mut manager = {
            let config_path = match cfg.sys_config {
                Some(ref path) => path.as_ref(),
                None => "/etc/openzwave",
            };

            let user_path = match cfg.user_config {
                Some(ref path) => path.as_ref(),
                None => "./config",
            };

            let opts = Options::create(config_path,
                                       user_path,
                                       "--SaveConfiguration true --DumpTriggerLevel 0 \
                                        --ConsoleOutput false")?;

            ozw::manager::Manager::create(opts)?
        };

        let devices = cfg.port.clone().map(|p| vec![p]).unwrap_or(get_default_devices());
        for device in devices {
            fs::File::open(&device)?;

            manager.add_driver(&device)?;
        }

        let manager = Arc::new(Mutex::new(manager));
        let items = Arc::new(Mutex::new(Default::default()));

        let (tx, rx) = channel(handle)?;

        let driver = ZWave {
            ozw_manager: manager.clone(),
            items: items,
        };

        let watcher = Watcher {
            cfg: cfg,
            driver: driver.clone(),
            output: Mutex::new(tx),
        };

        always_lock(manager.lock()).add_watcher(watcher)?;

        Ok((driver, rx))
    }

    pub fn get_manager(&self) -> MutexGuard<Manager> {
        always_lock(self.ozw_manager.lock())
    }
}

impl Binding for ZWave {
    type Config = Config;
    type Error = Error;
    type Item = Item;

    fn new(handle: &Handle, cfg: &Self::Config) -> Result<(Self, Receiver<Notification<Item>>)> {
        ZWave::new(handle, cfg)
    }

    fn get_value(&self, name: &str) -> BoxFuture<Option<Item>, Error> {
        futures::finished(always_lock(self.items.lock())
                .get_item(&String::from(name))
                .map(|i| i.clone()))
            .boxed()
    }
}

struct Watcher {
    driver: ZWave,
    cfg: Config,
    output: Mutex<Sender<Notification<Item>>>,
}

impl Watcher {
    fn get_out(&self) -> MutexGuard<Sender<Notification<Item>>> {
        ::catt_core::util::always_lock(self.output.lock())
    }
}

impl ozw::manager::NotificationWatcher for Watcher {
    fn on_notification(&self, zwave_notification: &ZWaveNotification) {
        let notification: Notification<Item> = match zwave_notification.get_type() {
            NotificationType::Type_DriverReady => {
                let home_id = zwave_notification.get_home_id();
                let controller = Item::controller(&format!("zwave_{}_Controller", home_id),
                                                  self.driver.clone(),
                                                  home_id);
                always_lock(self.driver.items.lock())
                    .add_item(controller.get_name(), controller.clone());
                let _ = self.get_out().send(Notification::Added(controller.clone()));
                Notification::Changed(controller)
            }
            NotificationType::Type_AllNodesQueried |
            NotificationType::Type_AwakeNodesQueried |
            NotificationType::Type_AllNodesQueriedSomeDead => {
                debug!("Controller ready");
                // self.driver.ozw_manager.write_configs();
                return;
            }

            NotificationType::Type_ValueAdded => {
                let v = zwave_notification.get_value_id();
                if !should_expose(v) {
                    return;
                }
                let mut db = always_lock(self.driver.items.lock());
                let (name, exists) = match self.cfg.lookup_device(v) {
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
                        if self.cfg.expose_unbound.unwrap_or(true) {
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
                            return;
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

            NotificationType::Type_ValueChanged => {
                let v = zwave_notification.get_value_id();
                if !should_expose(v) {
                    return;
                }
                let db = always_lock(self.driver.items.lock());
                let name = match db.get_name(&v) {
                    Some(n) => n,
                    None => return,
                };
                debug!("value {} changed: {:?}", name, v);
                let item = Item::item(&name, v);
                Notification::Changed(item)
            }

            NotificationType::Type_ValueRemoved => {
                let v = zwave_notification.get_value_id();
                if !should_expose(v) {
                    return;
                }
                let mut db = always_lock(self.driver.items.lock());
                let name = match db.get_name(&v) {
                    Some(n) => n.clone(),
                    None => return,
                };
                debug!("removing value {} from db", name);
                Notification::Removed(match db.remove_value(v) {
                    Some(it) => it,
                    None => Item::item(&name, v),
                })
            }

            // TODO new implementation for this
            // ZWaveNotification::Generic(s) => {
            //     if s.contains("Type_DriverRemoved") {
            //         warn!("controller removed! shutting down.");
            //         ::std::process::exit(1);
            //     }
            //     return;
            // }
            NotificationType::Type_ControllerCommand => {
                let home_id = zwave_notification.get_home_id();
                let db_name = format!("zwave_{}_Controller", home_id);
                let controller = match always_lock(self.driver.items.lock()).get_item(&db_name) {
                    Some(c) => c.clone(),
                    None => {
                        debug!("controller not found in item db");
                        return;
                    }
                };

                let state = match ControllerState::from_u8(zwave_notification.get_event()
                    .unwrap()) {
                    Some(s) => s,
                    None => {
                        debug!("unknown controller state: {}",
                               zwave_notification.get_event().unwrap());
                        return;
                    }
                };

                match state {
                    ControllerState::Completed => {
                        let _ = controller.set_value(Value::String("idle".into()));
                    }
                    ControllerState::Failed => {
                        let _ = controller.set_value(Value::String("failed".into()));
                    }
                    ControllerState::Starting => {}
                    _ => {
                        debug!("unhandled controller state: {:?}", state);
                        return;
                    }
                }
                Notification::Changed(controller)
            }

            _ => {
                debug!("unmatched notification: {}", zwave_notification);
                return;
            }
        };

        match self.get_out().send(notification) {
            Ok(_) => {}
            Err(e) => {
                warn!("zwave notification send error: {}", e);
                return;
            }
        }
    }
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
