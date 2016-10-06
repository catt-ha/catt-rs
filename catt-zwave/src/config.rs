use openzwave::value_classes::value_id::ValueID;
use openzwave::value_classes::value_id::ValueType;
use openzwave::value_classes::value_id::ValueGenre;

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
    pub name: String,
    pub id: u64,
    pub command_class: Option<String>,
    pub value_type: Option<String>,
    pub zwave_label: Option<String>,
}

impl DeviceConfig {
    fn matches(&self, value_id: ValueID) -> bool {
        if value_id.get_genre() != ValueGenre::ValueGenre_User {
            return false;
        }

        if value_id.get_node_id() as u64 != self.id {
            return false;
        }

        if let Some(ref cc_str) = self.command_class {
            if !cc_match(&cc_str, value_id) {
                return false;
            }
        }

        if let Some(ref vt_str) = self.value_type {
            if !vt_match(&vt_str, value_id) {
                return false;
            }
        }

        if let Some(ref label) = self.zwave_label {
            if !(label.to_lowercase() == value_id.get_label().to_lowercase()) {
                return false;
            }
        }

        true
    }
}

fn cc_match(cc_str: &str, value_id: ValueID) -> bool {
    let value_class = match value_id.get_command_class() {
        Some(c) => c,
        None => return false,
    };
    cc_str.to_lowercase() == format!("{}", value_class).to_lowercase()
}

fn vt_match(vt_str: &str, value_id: ValueID) -> bool {
    let value_type = match value_id.get_type() {
        ValueType::ValueType_Bool => "bool",
        ValueType::ValueType_Int |
        ValueType::ValueType_Byte |
        ValueType::ValueType_Short => "int",
        ValueType::ValueType_Decimal => "float",
        ValueType::ValueType_String => "string",
        _ => return false,
    };
    vt_str.to_lowercase() == value_type
}

#[derive(Eq,PartialEq)]
struct DeviceMatch<'a> {
    strength: u8,
    name: &'a str,
}

impl<'a> PartialOrd for DeviceMatch<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        Some(self.strength.cmp(&other.strength))
    }
}

impl<'a> Ord for DeviceMatch<'a> {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Config {
    pub fn lookup_device(&self, value_id: ValueID) -> Option<String> {
        let cfg = self.device
            .iter()
            .filter(|cfg| cfg.matches(value_id))
            .nth(0);

        if cfg.is_none() {
            return None;
        }

        let cfg = cfg.unwrap();

        return Some(cfg.name.clone());
    }
}
