use openzwave_stateful::ValueID;
use openzwave_stateful::ValueType;
use openzwave_stateful::ValueGenre;

#[derive(RustcDecodable,Debug,Clone,Default)]
pub struct ZWaveConfig {
    pub port: Option<String>,
    pub sys_config: Option<String>,
    pub user_config: Option<String>,
    pub device: Vec<DeviceConfig>,
    pub expose_unbound: bool,
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
    fn match_strength(&self, value_id: ValueID) -> u8 {
        if value_id.get_genre() != ValueGenre::ValueGenre_User {
            return 0;
        }

        if value_id.get_node_id() as u64 != self.id {
            return 0;
        }

        let mut strength = 1;

        match self.command_class {
            Some(ref cc_str) => {
                if cc_match(cc_str, value_id) {
                    strength += 1;
                } else {
                    return 0;
                }
            }
            None => {}
        };

        match self.value_type {
            Some(ref vt_str) => {
                if vt_match(vt_str, value_id) {
                    strength += 1;
                } else {
                    return 0;
                }
            }
            None => {}
        };

        match self.zwave_label {
            Some(ref label) => {
                if label.to_lowercase() == value_id.get_label().to_lowercase() {
                    debug!("label matched!");
                    strength += 1;
                } else {
                    return 0;
                }
            }
            None => {}
        };

        strength
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

impl ZWaveConfig {
    pub fn lookup_device(&self, value_id: ValueID) -> Option<(String, u8)> {
        let best = self.device
            .iter()
            .map(|cfg| {
                let strength = cfg.match_strength(value_id);
                DeviceMatch {
                    name: &cfg.name,
                    strength: strength,
                }
            })
            .max();

        if best.is_none() {
            return None;
        }

        let best = best.unwrap();

        if best.strength != 0 {
            return Some((best.name.into(), best.strength));
        }

        None
    }
}
