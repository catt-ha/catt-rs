use openzwave::value_classes::value_id::ValueID;

#[derive(RustcDecodable,Debug,Clone,Default)]
pub struct Config {
    pub port: Option<String>,
    pub sys_config: Option<String>,
    pub user_config: Option<String>,
    pub device: Vec<DeviceConfig>,
    pub expose_unbound: Option<bool>,
}

#[derive(RustcDecodable,Debug,Clone,Default)]
pub struct DeviceConfig {
    pub id: u64,
    pub name: String,
}

impl DeviceConfig {
    fn matches(&self, value_id: ValueID) -> bool {
        if value_id.get_node_id() as u64 != self.id {
            return false;
        }

        true
    }
}

impl Config {
    pub fn lookup_device(&self, value_id: ValueID) -> Option<String> {
        let cfg = self.device
            .iter()
            .filter(|cfg| cfg.matches(value_id))
            .nth(0);

        cfg.map(|c| format!("{}_{}", c.name, value_id.get_label()))
    }
}
