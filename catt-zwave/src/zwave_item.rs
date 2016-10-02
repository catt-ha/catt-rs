use openzwave_stateful as ozw;
use openzwave_stateful::ValueType;

use catt_core::item;
use catt_core::value::Value as CValue;

use std::collections::HashMap;

use super::errors::*;

#[derive(PartialOrd, Ord, Eq, PartialEq, Debug, Clone)]
pub struct ZWaveItem {
    name: String,
    ozw_value: ozw::ValueID,
}

impl ZWaveItem {
    pub fn new(name: &str, value: ozw::ValueID) -> Self {
        ZWaveItem {
            name: name.into(),
            ozw_value: value,
        }
    }

    pub fn set_number(&self, number: f64) -> Result<()> {
        let val_type = self.ozw_value.get_type();
        let res = match val_type {
            ValueType::ValueType_Byte => self.ozw_value.set_byte(number as u8),
            ValueType::ValueType_Short => self.ozw_value.set_short(number as i16),
            ValueType::ValueType_Int => self.ozw_value.set_int(number as i32),
            ValueType::ValueType_Decimal => self.ozw_value.set_float(number as f32),
            _ => {
                // TODO real "actual" type string
                return Err(ErrorKind::InvalidType(self.name.clone(), "float", "not a number")
                    .into());
            }
        };
        Ok(res?)
    }

    pub fn set_bool(&self, val: bool) -> Result<()> {
        Ok(self.ozw_value.set_bool(val)?)
    }

    pub fn set_string(&self, val: &str) -> Result<()> {
        Ok(self.ozw_value.set_string(val)?)
    }

    pub fn set_raw(&self, val: &Vec<u8>) -> Result<()> {
        Ok(self.ozw_value.set_raw(val)?)
    }
}

impl item::Item for ZWaveItem {
    type Error = Error;

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_value(&self) -> Result<CValue> {
        let val_type = self.ozw_value.get_type();
        let val = match val_type {
            ValueType::ValueType_Button |
            ValueType::ValueType_List |
            ValueType::ValueType_Schedule => {
                return Err(ErrorKind::Unimplemented(self.get_name(), val_type).into())
            }

            ValueType::ValueType_Raw => CValue::Raw(*self.ozw_value.as_raw()?),

            ValueType::ValueType_Bool => CValue::Bool(self.ozw_value.as_bool()?),
            ValueType::ValueType_Byte => CValue::Number(self.ozw_value.as_byte()? as f64),
            ValueType::ValueType_Short => CValue::Number(self.ozw_value.as_short()? as f64),
            ValueType::ValueType_Int => CValue::Number(self.ozw_value.as_int()? as f64),
            ValueType::ValueType_Decimal => CValue::Number(self.ozw_value.as_float()? as f64),
            ValueType::ValueType_String => CValue::String(self.ozw_value.as_string()?),

        };

        Ok(val)
    }

    fn get_meta(&self) -> Option<item::Meta> {
        let mut ext = HashMap::new();
        ext.insert("label".into(), self.ozw_value.get_label());
        ext.insert("node_id".into(),
                   format!("{}", self.ozw_value.get_node_id()));
        ext.insert("command_class".into(),
                   self.ozw_value
                       .get_command_class()
                       .map(|cc| format!("{:?}", cc))
                       .unwrap_or("???".into()));

        Some(item::Meta {
            backend: String::from("zwave").into(),
            value_type: match self.get_value() {
                    Ok(v) => String::from(v.type_string()),
                    Err(_) => return None,
                }
                .into(),
            ext: ext.into(),
        })
    }

    fn set_value(&self, value: CValue) -> Result<()> {
        let val_type = self.ozw_value.get_type();
        let res = match val_type {
            ValueType::ValueType_Raw => self.set_raw(&value.as_raw()?),

            ValueType::ValueType_Button |
            ValueType::ValueType_List |
            ValueType::ValueType_Schedule => {
                return Err(ErrorKind::Unimplemented(self.get_name(), val_type).into())
            }

            ValueType::ValueType_Bool => self.set_bool(value.as_bool()?),
            ValueType::ValueType_Byte |
            ValueType::ValueType_Short |
            ValueType::ValueType_Int |
            ValueType::ValueType_Decimal => self.set_number(value.as_number()?),
            ValueType::ValueType_String => self.set_string(&value.as_string()?),
        };

        Ok(res?)
    }
}
