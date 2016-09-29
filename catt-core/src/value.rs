use rustc_serialize::base64::{self, STANDARD, FromBase64, ToBase64};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use std::io;
use std::io::Cursor;
use std::num;
use std::str;

error_chain!{
    foreign_links {
        io::Error, Io;
        num::ParseFloatError, ParseNumber;
        base64::FromBase64Error, FromBase64;
    }

    errors {
        InvalidConversion(v: Value, t: &'static str) {
            description("invalid type conversion")
            display("could not convert {:?} to type {}", v, t)
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Raw(Vec<u8>),
    Bool(bool),
}

impl Value {
    pub fn from_raw(v: &[u8]) -> Value {
        let value = match String::from_utf8(v.into()) {
            Ok(s) => Value::String(s),
            Err(_) => Value::Raw(v.into()),
        };

        un_stringify(value)
    }

    pub fn as_number(&self) -> Result<f64> {
        Ok(match self {
            &Value::Number(ref n) => *n,
            &Value::String(ref s) => s.trim().parse()?,
            &Value::Raw(ref v) => Cursor::new(v).read_f64::<BigEndian>()?,
            &Value::Bool(ref b) => if *b { 1.0 } else { 0.0 },
        })
    }

    pub fn as_number_value(&self) -> Result<Value> {
        Ok(Value::Number(self.as_number()?))
    }

    pub fn as_string(&self) -> Result<String> {
        Ok(match self {
            &Value::Number(ref n) => format!("{}", n),
            &Value::String(ref s) => s.clone(),
            &Value::Raw(ref v) => v.to_base64(STANDARD),
            &Value::Bool(ref b) => if *b { "ON".into() } else { "OFF".into() },
        })
    }

    pub fn as_string_value(&self) -> Result<Value> {
        Ok(Value::String(self.as_string()?))
    }

    pub fn as_bool(&self) -> Result<bool> {
        Ok(match self {
            &Value::Number(ref n) => *n as i64 != 0,
            &Value::String(ref s) => {
                match s.trim().to_lowercase().as_str() {
                    "on" | "open" | "true" => true,
                    "off" | "closed" | "false" => false,
                    _ => return Err(ErrorKind::InvalidConversion(self.clone(), "bool").into()),
                }
            }
            &Value::Raw(ref v) => if v.len() > 0 { v[0] != 0u8 } else { false },
            &Value::Bool(ref b) => *b,
        })
    }

    pub fn as_bool_value(&self) -> Result<Value> {
        Ok(Value::Bool(self.as_bool()?))
    }

    pub fn as_raw(&self) -> Result<Vec<u8>> {
        Ok(match self {
            &Value::Number(ref n) => {
                let mut wr = vec![];
                wr.write_f64::<BigEndian>(*n)?;
                wr
            }
            &Value::String(ref s) => s.from_base64()?,
            &Value::Raw(ref v) => v.clone(),
            &Value::Bool(ref b) => vec![*b as u8],
        })
    }

    pub fn as_raw_value(&self) -> Result<Value> {
        Ok(Value::Raw(self.as_raw()?))
    }
}

fn un_stringify(val: Value) -> Value {
    match val {
        Value::String(_) => {}
        _ => return val,
    }

    match val.as_bool_value() {
        Ok(v) => return v,
        _ => {}
    }

    match val.as_number_value() {
        Ok(v) => return v,
        _ => {}
    }

    val
}
