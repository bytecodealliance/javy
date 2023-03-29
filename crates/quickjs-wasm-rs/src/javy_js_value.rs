use anyhow::{anyhow, Result};
use std::convert::TryFrom;
use std::{collections::HashMap, fmt};

// Should this type be in a completely separate crate if we plan to have multiple JS engines?
// That way the spidermonkey engine can also use to serialize to their internal types

#[derive(Debug, Clone)]
pub enum JavyJSValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32), // do we need to support i8..i64?
    Float(f64),
    String(String),
    Array(Vec<JavyJSValue>),
    Object(HashMap<String, JavyJSValue>),
    ArrayBuffer(Vec<u8>),
    MutArrayBuffer(*mut u8, usize), // hacky? used for readSync. need to hold a raw pointer to qjs memory to write directly to
}

impl JavyJSValue {    
    // pub fn as_bytes(&self) -> Result<&[u8]> {
    //     match self {
    //         JavyJSValue::MutArrayBuffer(bytes) => Ok(bytes.as_slice()),
    //         _ => Err(anyhow!("Can't represent as an array buffer")),
    //     }
    // }

    pub fn as_bytes_mut(&self) -> Result<&mut [u8]> {
        match self {
            JavyJSValue::MutArrayBuffer(bytes, len) => {
                let bytes = unsafe { std::slice::from_raw_parts_mut(*bytes, *len) };
                Ok(bytes)
            },
            _ => Err(anyhow!("Can't represent as an array buffer")),
        }
    }

    pub fn as_i32(&self) -> Result<i32> {
        match self {
            JavyJSValue::Int(i) => Ok(*i),
            _ => Err(anyhow!("Can't represent as i32")),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match self {
            JavyJSValue::Float(n) => Ok(*n),
            _ => Err(anyhow!("Can't represent as f64")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            JavyJSValue::Bool(b) => Ok(*b),
            _ => Err(anyhow!("Can't represent as a bool")),
        }
    }

    // pub fn try_as_integer(&self) -> Result<i32> {
    //     match self {
    //         JavyJSValue::Int(i) => Ok(*i),
    //         JavyJSValue::Float(n) => Ok(i32::try_from(*n)?),
    //         _ => Err(anyhow!("Can't represent as an integer")),
    //     }
    // }
}

// Used http://numcalc.com/ to playaround and determine the default display format for each type
impl fmt::Display for JavyJSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JavyJSValue::Undefined => write!(f, "undefined"),
            JavyJSValue::Null => write!(f, "null"),
            JavyJSValue::Bool(b) => write!(f, "{}", b),
            JavyJSValue::Int(i) => write!(f, "{}", i),
            JavyJSValue::Float(n) => write!(f, "{}", n),
            JavyJSValue::String(s) => write!(f, "{}", s),
            JavyJSValue::MutArrayBuffer(_, _) => write!(f, "ArrayBuffer"),
            JavyJSValue::ArrayBuffer(buffer) => write!(f, "{:?}", buffer),
            JavyJSValue::Array(arr) => {
                write!(f, "{}", arr.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(","))     
            }
            JavyJSValue::Object(_) => write!(f, "[object Object]"),
        }
    }
}

// impl TryFrom<JavyJSValue> for Vec<u8> {
//     type Error = anyhow::Error;
//     fn try_from(value: JavyJSValue) -> Result<Self, Self::Error> {
//         match value {
//             JavyJSValue::ArrayBuffer(bytes) => Ok(bytes),
//             _ => Err(anyhow!("Can't represent as an array buffer")),
//         }
//     }
// }

impl TryFrom<JavyJSValue> for i32 {
    type Error = anyhow::Error;
    fn try_from(value: JavyJSValue) -> Result<Self, Self::Error> {
        match value {
            JavyJSValue::Int(i) => Ok(i),
            _ => Err(anyhow!("Can't represent as an i32")),
        }
    }
}

impl TryFrom<JavyJSValue> for bool {
    type Error = anyhow::Error;
    fn try_from(value: JavyJSValue) -> Result<Self, Self::Error> {
        match value {
            JavyJSValue::Bool(b) => Ok(b),
            _ => Err(anyhow!("Can't represent as a bool")),
        }
    }
}