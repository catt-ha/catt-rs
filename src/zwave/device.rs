use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use config::DeviceConfig;

#[derive(Default,Debug)]
pub struct DeviceDB {
    by_name: HashMap<String, Rc<RefCell<Device>>>,
    by_id: HashMap<DeviceDesc, Rc<RefCell<Device>>>,
}

#[derive(Default,Debug)]
pub struct Device {
    name: String,
    desc: DeviceDesc,
    state: DeviceState,
}

#[derive(Default,Debug,Clone,PartialEq,Eq,Hash)]
pub struct DeviceDesc {
    id: u64,
    endpoint: Option<u64>,
    command: String,
}

#[derive(Debug)]
pub enum DeviceState {
    Basic(i64),
    BinarySwitch(bool),
    BinarySensor(bool),
    Multilevel(i64),
}

impl Default for DeviceState {
    fn default() -> Self {
        return DeviceState::Basic(0);
    }
}

impl<'a> From<&'a str> for DeviceState {
    fn from(other: &'a str) -> Self {
        match other {
            "basic" => DeviceState::Basic(0),
            "switch_binary" => DeviceState::BinarySwitch(false),
            "sensor_binary" => DeviceState::BinarySensor(false),
            "switch_multilevel" => DeviceState::Multilevel(0),
            _ => DeviceState::Basic(0),
        }
    }
}


impl DeviceDB {
    pub fn from_config(configs: Vec<DeviceConfig>) -> Self {
        let mut db: DeviceDB = Default::default();
        configs.iter()
            .map(|ref cfg| {
                let mut dev = Device::default();
                dev.name = cfg.name.clone();
                {
                    let desc = &mut dev.desc;
                    desc.id = cfg.id;
                    desc.endpoint = cfg.endpoint;
                    desc.command = cfg.command
                        .as_ref()
                        .map_or("basic".into(), |cmd| cmd.clone());
                }
                cfg.command.as_ref().map(|ref cmd| dev.state = DeviceState::from(cmd.as_ref()));
                let shared = Rc::new(RefCell::new(dev));
                db.by_id.insert(shared.borrow().desc.clone(), shared.clone());
                db.by_name.insert(shared.borrow().name.clone(), shared.clone());
            })
            .collect::<Vec<()>>();
        db
    }
}
