use openzwave_stateful::ValueID;

use std::collections::HashMap;

use config::DeviceConfig;

#[derive(Default,Debug,Clone)]
pub struct DeviceDB {
    pub by_name: HashMap<String, Device>,
    pub by_value: HashMap<ValueID, Device>,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub config: DeviceConfig,
    pub value: Option<ValueID>,
}

#[derive(Debug)]
pub enum DeviceType {
    Basic,
    BinarySwitch,
    BinarySensor,
    Multilevel,
}

impl DeviceDB {
    pub fn from_config(configs: &[DeviceConfig]) -> Self {
        let mut db: DeviceDB = Default::default();
        configs.iter()
            .cloned()
            .map(|cfg| {
                let name = cfg.name.clone();

                let dev = Device {
                    config: cfg,
                    value: None,
                };

                db.by_name.insert(name, dev);
            })
            .collect::<Vec<()>>();
        db
    }
}

pub fn gen_name(val: &ValueID) -> String {
    format!("zwave_{}_{}_{}",
            val.get_node_id(),
            val.get_command_class().map_or("???".to_string(), |c| c.to_string()),
            val.get_index(),
    )
}

pub fn gen_device(val: &ValueID) -> Device {
    let cfg = DeviceConfig {
        name: gen_name(val),
        id: val.get_node_id() as u64,
        genre: None,
        command_class: None,
        value_type: None,
        index: None,
    };
    Device {
        value: Some(*val),
        config: cfg,
    }
}
