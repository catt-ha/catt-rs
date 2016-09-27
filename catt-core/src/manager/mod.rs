pub mod device;
pub mod errors;

use self::errors::*;

use std::collections::BTreeSet;
use std::sync::Arc;

use std::sync::Mutex;

use zwave::driver::Driver;
use zwave::driver::ValueID;
use zwave::class::CommandClass;
use zwave::class::ValueType;
use zwave::class::ZClass;

use openzwave_stateful::ValueGenre;

use bus::mqtt::Mqtt;

use self::device::*;

use config;

use mqtt3::Message;

pub struct Manager {
    zwave_driver: Driver,
    bus: Mqtt,
    devices: DeviceDB,
}

impl Manager {
    pub fn with_config(cfg: config::Config) -> Result<Arc<Mutex<Self>>> {
        let manager = Manager {
            zwave_driver: Driver::new(cfg.zwave.port.as_ref().map(|s| s.as_ref()),
                                      cfg.zwave.sys_config.as_ref().map(|s| s.as_ref()),
                                      cfg.zwave
                                          .user_config
                                          .as_ref()
                                          .map(|s| s.as_ref())
                                          .unwrap_or("./zwave_config"))?,
            bus: Mqtt::with_config(&cfg.mqtt)?,
            devices: Default::default(),
        };

        let manager = Arc::new(Mutex::new(manager));

        {
            let mut mg = ::util::always_lock(manager.lock());

            mg.zwave_driver.wait_ready();
            debug!("zwave driver ready");

            // does nothing currently
            mg.bind_devices(&cfg.zwave.device);

            if cfg.zwave.expose_unbound {
                mg.bind_unbound();
            }

            let devices = mg.devices.clone();
            mg.bus.subscribe(&devices)?;
        }

        zwave_thread(manager.clone());
        bus_thread(manager.clone());


        debug!("manager ready");

        Ok(manager)
    }

    pub fn get_device(&self, name: &str) -> Option<&Device> {
        self.devices.by_name.get(name)
    }

    pub fn get_devices(&self) -> &DeviceDB {
        &self.devices
    }

    pub fn get_driver(&mut self) -> &mut Driver {
        &mut self.zwave_driver
    }

    fn bind_unbound(&mut self) {
        let values: BTreeSet<ValueID> = self.get_driver()
            .state()
            .get_values()
            .iter()
            .cloned()
            .filter(|v| v.get_genre() == ValueGenre::ValueGenre_User)
            .collect();

        for v in values {
            let bound = self.devices.by_value.contains_key(&v);
            if !bound {
                let dev = gen_device(&v);
                self.devices.by_value.insert(v, dev.clone());
                self.devices.by_name.insert(dev.config.name.clone(), dev);
            }
        }
    }

    fn bind_devices(&mut self, cfg: &[config::DeviceConfig]) {}
}

fn bus_thread(manager: Arc<Mutex<Manager>>) {
    ::std::thread::spawn(move || {
        debug!("Spawned mqtt -> zwave thread.");
        loop {
            let recv_result = ::util::always_lock(manager.lock()).bus.get_message();
            let msg_opt = match recv_result {
                Ok(m) => m,
                Err(e) => {
                    warn!("{}", e);
                    continue;
                }
            };

            let msg: Box<Message> = match msg_opt {
                Some(m) => m,
                None => continue,
            };

            info!("{:?}", msg);
        }
        debug!("mqtt -> zwave thread died.")
    });
}

fn zwave_thread(manager: Arc<Mutex<Manager>>) {
    ::std::thread::spawn(move || {
        debug!("Spawned zwave -> mqtt thread.");
        loop {
            let mut manager = ::util::always_lock(manager.lock());
            let n_opt = manager.get_driver()
                .updates()
                .recv();

            let notification = match n_opt {
                Ok(n) => n,
                Err(e) => {
                    warn!("{:?}", e);
                    continue;
                }
            };

            if let Some(name) = manager.devices
                .by_value
                .get(&notification)
                .map(|d| &d.config.name) {
                match manager.bus
                    .publish(name, &notification.as_string().unwrap_or("???".into())) {
                    Ok(_) => {}
                    Err(e) => warn!("publish error: {:?}", e),
                };
            }
        }
        debug!("zwave -> mqtt thread died.")
    });
}

#[cfg(test)]
mod test {
    // use std::sync::{Arc, Mutex};
    // use config::Config;
    // use super::Manager;

    // type ThreadSharedManager = Arc<Mutex<Manager>>;

    // fn build_manager() -> Manager {
    //     let cfg = Config::from_file("config.toml").unwrap();
    //     let manager = Manager::with_config(cfg).unwrap();

    //     manager
    // }

    // #[test]
    // fn test_build_manager() {
    //     build_manager();
    // }
}
