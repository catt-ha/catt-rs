use util::CVar;

use openzwave_stateful::{self, ConfigPath, InitOptions, ZWaveManager, ZWaveNotification, ValueType};

pub use openzwave_stateful::ValueID;
pub use openzwave_stateful::State;

use std::sync::mpsc::{self, Receiver, Sender};

use std::sync::MutexGuard;
use std::iter::Iterator;
use std::sync::Arc;

use std::collections::BTreeSet;

use super::errors::*;

use super::device::Device;

pub struct Driver {
    manager: Arc<ZWaveManager>,
    ready: CVar,
    updates: Receiver<ValueID>,
    devices: Option<BTreeSet<Device>>,
}

impl Driver {
    pub fn new(device: Option<&str>, sys_config: Option<&str>, usr_config: &str) -> Result<Driver> {

        let (manager, notifications) = openzwave_stateful::init(&InitOptions {
            devices: device.map(|x| vec![x.into()]),
            config_path: sys_config.map(|x| ConfigPath::Custom(x)).unwrap_or(ConfigPath::Default),
            user_path: usr_config.into(),
        })?;

        let ready = CVar::new();

        let (tx, rx) = mpsc::channel();

        let driver = Driver {
            manager: manager,
            ready: ready.clone(),
            updates: rx,
            devices: None,
        };

        spawn_notification_thread(tx, notifications, ready);

        Ok(driver)
    }

    #[allow(unused_must_use)]
    pub fn wait_ready(&self) {
        self.ready.wait();
    }

    pub fn updates(&self) -> &Receiver<ValueID> {
        &self.updates
    }

    pub fn get_devices(&mut self) -> &BTreeSet<Device> {
        match self.devices {
            Some(ref d) => d,
            None => {
                self.init_devices();
                self.get_devices()
            }
        }
    }

    fn init_devices(&mut self) {
        let state = self.manager.get_state();

        let nodes = state.get_nodes();
        let values = state.get_values();
        let mut devices = BTreeSet::new();

        nodes.iter()
            .map(|n| {
                devices.insert(Device {
                    node: n.clone(),
                    values: values.iter()
                        .cloned()
                        .filter(|v| v.get_node_id() == n.get_id())
                        .collect(),
                })
            })
            .collect::<Vec<_>>();

        self.devices = Some(devices);
    }

    pub fn state(&self) -> MutexGuard<State> {
        self.manager.get_state()
    }
}

fn spawn_notification_thread(output: Sender<ValueID>,
                             rx: Receiver<ZWaveNotification>,
                             ready: CVar) {
    ::std::thread::spawn(move || {
        for not in rx {
            match not {
                ZWaveNotification::AllNodesQueried(_) |
                ZWaveNotification::AwakeNodesQueried(_) |
                ZWaveNotification::AllNodesQueriedSomeDead(_) => {
                    debug!("Controller ready");
                    ready.notify_all();
                }

                ZWaveNotification::ValueAdded(v) |
                ZWaveNotification::ValueRefreshed(v) |
                ZWaveNotification::ValueChanged(v) => {
                    output.send(v).unwrap();
                }
                n => {
                    debug!("unmatched notification: {}", n);
                }
            }
        }

    });
}
