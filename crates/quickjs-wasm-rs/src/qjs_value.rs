use anyhow::{anyhow, Result};
use std::convert::TryFrom;
use std::{collections::HashMap, fmt};

// Should this type be in a completely separate crate if we plan to have multiple JS engines?
// That way the spidermonkey engine can also use to serialize to their internal types

#[derive(Debug, Clone)]
pub enum QJSValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32), // do we need to support i8..i64?
    Float(f64),
    String(String),
    Array(Vec<QJSValue>),
    Object(HashMap<String, QJSValue>),
    ArrayBuffer(Vec<u8>),
    MutArrayBuffer(*mut u8, usize), // hacky? used for readSync. need to hold a raw pointer to qjs memory to write directly to
}

impl QJSValue {    
    // pub fn as_bytes(&self) -> Result<&[u8]> {
    //     match self {
    //         QJSValue::MutArrayBuffer(bytes) => Ok(bytes.as_slice()),
    //         _ => Err(anyhow!("Can't represent as an array buffer")),
    //     }
    // }

    pub fn as_bytes_mut(&self) -> Result<&mut [u8]> {
        match self {
            QJSValue::MutArrayBuffer(bytes, len) => {
                let bytes = unsafe { std::slice::from_raw_parts_mut(*bytes, *len) };
                Ok(bytes)
            },
            _ => Err(anyhow!("Can't represent as an array buffer")),
        }
    }

    pub fn as_i32(&self) -> Result<i32> {
        match self {
            QJSValue::Int(i) => Ok(*i),
            _ => Err(anyhow!("Can't represent as i32")),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match self {
            QJSValue::Float(n) => Ok(*n),
            _ => Err(anyhow!("Can't represent as f64")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            QJSValue::Bool(b) => Ok(*b),
            _ => Err(anyhow!("Can't represent as a bool")),
        }
    }

    // pub fn try_as_integer(&self) -> Result<i32> {
    //     match self {
    //         QJSValue::Int(i) => Ok(*i),
    //         QJSValue::Float(n) => Ok(i32::try_from(*n)?),
    //         _ => Err(anyhow!("Can't represent as an integer")),
    //     }
    // }
}

// Used http://numcalc.com/ to playaround and determine the default display format for each type
impl fmt::Display for QJSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QJSValue::Undefined => write!(f, "undefined"),
            QJSValue::Null => write!(f, "null"),
            QJSValue::Bool(b) => write!(f, "{}", b),
            QJSValue::Int(i) => write!(f, "{}", i),
            QJSValue::Float(n) => write!(f, "{}", n),
            QJSValue::String(s) => write!(f, "{}", s),
            QJSValue::MutArrayBuffer(_, _) => write!(f, "ArrayBuffer"),
            QJSValue::ArrayBuffer(buffer) => write!(f, "{:?}", buffer),
            QJSValue::Array(arr) => {
                write!(f, "{}", arr.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(","))     
            }
            QJSValue::Object(_) => write!(f, "[object Object]"),
        }
    }
}

// impl TryFrom<QJSValue> for Vec<u8> {
//     type Error = anyhow::Error;
//     fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
//         match value {
//             QJSValue::ArrayBuffer(bytes) => Ok(bytes),
//             _ => Err(anyhow!("Can't represent as an array buffer")),
//         }
//     }
// }

impl TryFrom<QJSValue> for i32 {
    type Error = anyhow::Error;
    fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
        match value {
            QJSValue::Int(i) => Ok(i),
            _ => Err(anyhow!("Can't represent as an i32")),
        }
    }
}

impl TryFrom<QJSValue> for bool {
    type Error = anyhow::Error;
    fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
        match value {
            QJSValue::Bool(b) => Ok(b),
            _ => Err(anyhow!("Can't represent as a bool")),
        }
    }
}