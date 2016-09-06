pub mod driver;
mod device;
mod errors;
use self::device::*;


use config;

#[derive(Debug)]
pub struct Manager {
    devices: DeviceDB,
}

impl Manager {
    fn with_config(cfg: config::Config) -> Self {
        Manager { devices: DeviceDB::from_config(cfg.zwave.device) }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use config::Config;
    use super::Manager;

    type ThreadSharedManager = Arc<Mutex<Manager>>;

    fn build_manager() -> Manager {
        let cfg = Config::from_file("config.toml").unwrap();
        let manager = Manager::with_config(cfg);

        manager
    }

    #[test]
    fn test_build_manager() {
        build_manager();
    }
}
