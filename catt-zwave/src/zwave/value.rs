use openzwave_stateful as ozw;
use openzwave_stateful::ValueType;
use catt_core::binding;

use super::errors::*;

#[derive(PartialOrd, Ord, Eq, PartialEq, Debug, Clone)]
pub struct Value {
    name: String,
    ozw_value: ozw::ValueID,
}

impl Value {
    pub fn new(name: &str, value: ozw::ValueID) -> Self {
        Value {
            name: name.into(),
            ozw_value: value,
        }
    }

    pub fn set_number(&self, value: &str) -> Result<()> {
        let number = to_float(value)?;
        let val_type = self.ozw_value.get_type();
        let res = match val_type {
            ValueType::ValueType_Byte => self.ozw_value.set_byte(number as u8),
            ValueType::ValueType_Short => self.ozw_value.set_short(number as i16),
            ValueType::ValueType_Int => self.ozw_value.set_int(number as i32),
            ValueType::ValueType_Decimal => self.ozw_value.set_float(number as f32),
            _ => unreachable!(),
        };
        Ok(res?)
    }

    pub fn set_string(&self, val: &str) -> Result<()> {
        Ok(self.ozw_value.set_string(val)?)
    }

    pub fn set_raw(&self, val: &[u8]) -> Result<()> {
        Ok(self.ozw_value.set_raw(&val.into())?)
    }
}

impl binding::Value for Value {
    type Error = Error;

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_value(&self) -> Result<Vec<u8>> {
        match self.ozw_value.as_string().map(|v| v.into_bytes()) {
            Ok(res) => Ok(res),
            Err(_) => Ok(*self.ozw_value.as_raw()?),
        }
    }

    fn set_value(&self, value: &[u8]) -> Result<()> {
        if self.ozw_value.get_type() == ValueType::ValueType_Raw {
            return Ok(self.set_raw(value)?);
        }

        let val_vec = value.into();
        let val_str = String::from_utf8(val_vec)?;

        let val_type = self.ozw_value.get_type();
        let res = match val_type {
            ValueType::ValueType_Raw => unreachable!(),

            ValueType::ValueType_Button |
            ValueType::ValueType_List |
            ValueType::ValueType_Schedule => {
                return Err(ErrorKind::Unimplemented(self.get_name(), val_type).into())
            }

            ValueType::ValueType_Bool |
            ValueType::ValueType_Byte |
            ValueType::ValueType_Short |
            ValueType::ValueType_Int |
            ValueType::ValueType_Decimal => self.set_number(&val_str),
            ValueType::ValueType_String => self.set_string(&val_str),
        };

        Ok(res?)
    }
}

fn to_float(s: &str) -> Result<f64> {
    let s = s.trim();

    let res: ::std::result::Result<f64, _> = s.parse();

    if res.is_ok() {
        return Ok(res.unwrap());
    }

    let res = match s.to_lowercase().as_str() {
        "on" | "open" | "true" => Ok(99.0),
        "off" | "closed" | "false" => Ok(0.0),
        _ => Err("WIP".into()),
    };

    debug!("number: {:?}", res);
    res
}
