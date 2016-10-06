pub use openzwave::value_classes::value_id::CommandClass as ZClass;
pub use openzwave::value_classes::value_id::ValueType as ZType;

pub struct CommandClass(ZClass);
use std::ops::Deref;

impl CommandClass {
    pub fn from_str(s: &str) -> Option<CommandClass> {
        Some(CommandClass(match s.to_lowercase().as_ref() {
            "basic" => ZClass::Basic,
            "switch_binary" => ZClass::SwitchBinary,
            "switch_multilevel" => ZClass::SwitchMultilevel,
            "sensor_binary" => ZClass::SensorBinary,
            "sensor_multilevel" => ZClass::SensorMultilevel,
            _ => {
                debug!("invalid command class: {}", s);
                return None;
            }
        }))
    }
    pub fn from_str_default(s: &str) -> CommandClass {
        CommandClass::from_str(s).unwrap_or(CommandClass(ZClass::Basic))
    }
}

impl Deref for CommandClass {
    type Target = ZClass;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone,Copy)]
pub struct ValueType(ZType);

impl ValueType {
    pub fn from_str(s: &str) -> Option<ValueType> {
        Some(ValueType(match s.to_lowercase().as_ref() {
            "bool" => ZType::ValueType_Bool,
            "byte" => ZType::ValueType_Byte,
            "decimal" => ZType::ValueType_Decimal,
            "int" => ZType::ValueType_Int,
            "short" => ZType::ValueType_Short,
            "string" => ZType::ValueType_String,
            _ => {
                debug!("invalid command class: {}", s);
                return None;
            }
        }))
    }
    pub fn from_str_default(s: &str) -> ValueType {
        ValueType::from_str(s).unwrap_or(ValueType(ZType::ValueType_Raw))
    }
}

impl Deref for ValueType {
    type Target = ZType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
