use cvar::CVar;

use openzwave_stateful::{self, ConfigPath, InitOptions, ValueID, ZWaveManager, ZWaveNotification};

use mioco::sync::mpsc::{self, Receiver, Sender};

use std::iter::Iterator;
use std::sync::Arc;

use super::errors::*;

use recv_wrap;

pub struct Driver {
    manager: Arc<ZWaveManager>,
    ready: CVar,
    updates: Receiver<ValueID>,
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
        };

        spawn_notification_thread(tx, recv_wrap::Receiver::new(notifications), ready);

        Ok(driver)
    }

    #[allow(unused_must_use)]
    pub fn wait_ready(&self) {
        self.ready.wait();
    }

    pub fn updates(&self) -> &Receiver<ValueID> {
        &self.updates
    }

    pub fn find_value(&self,
                      node_id: Option<u8>,
                      command_class: Option<u8>,
                      instance: Option<u8>,
                      index: Option<u8>)
                      -> Vec<ValueID> {
        let state = self.manager.get_state();

        let cloned_values = state.get_values()
            .iter()
            .cloned();

        let filtered_values =
            cloned_values.filter(|v| node_id.map_or(true, |i| v.get_node_id() == i))
                .filter(|v| command_class.map_or(true, |c| v.get_command_class_id() == c))
                .filter(|v| instance.map_or(true, |i| v.get_instance() == i))
                .filter(|v| index.map_or(true, |i| v.get_index() == i));

        filtered_values.collect()
    }
}

fn spawn_notification_thread(output: Sender<ValueID>,
                             rx: recv_wrap::Receiver<ZWaveNotification>,
                             ready: CVar) {
    ::mioco::spawn(move || {
        for not in rx {
            match not {
                ZWaveNotification::AllNodesQueried(_) |
                ZWaveNotification::AwakeNodesQueried(_) |
                ZWaveNotification::AllNodesQueriedSomeDead(_) => {
                    debug!("Controller ready");
                    ready.notify_all();
                }

                ZWaveNotification::ValueAdded(v) |
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
