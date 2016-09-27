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
        let val_vec = value.into();
        if self.ozw_value.get_type() == ValueType::ValueType_Raw {
            return Ok(self.ozw_value.set_raw(&val_vec)?);
        }

        let val_str = String::from_utf8(val_vec)?;

        let val_type = self.ozw_value.get_type();
        let res = match val_type {
            ValueType::ValueType_Raw => unreachable!(),

            ValueType::ValueType_Button |
            ValueType::ValueType_List |
            ValueType::ValueType_Schedule => {
                return Err(ErrorKind::Unimplemented(self.get_name(), val_type).into())
            }

            ValueType::ValueType_Bool => self.ozw_value.set_bool(val_str.parse()?),
            ValueType::ValueType_Byte => self.ozw_value.set_byte(val_str.parse()?),
            ValueType::ValueType_Short => self.ozw_value.set_short(val_str.parse()?),
            ValueType::ValueType_Int => self.ozw_value.set_int(val_str.parse()?),
            ValueType::ValueType_Decimal => self.ozw_value.set_float(val_str.parse()?),
            ValueType::ValueType_String => self.ozw_value.set_string(&val_str),
        };

        Ok(res?)
    }
}
