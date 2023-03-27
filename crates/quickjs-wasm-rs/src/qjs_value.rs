use anyhow::{anyhow, Result};
use std::convert::TryFrom;
use std::{collections::HashMap};

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
    ArrayBuffer(Vec<u8>),
    Array(Vec<QJSValue>),
    Object(HashMap<String, QJSValue>),
}

impl QJSValue {
    pub fn as_str(&self) -> Result<&str> {
        match self {
            QJSValue::String(s) => Ok(s.as_str()),
            _ => Err(anyhow!("Can't represent as an str")),
        }
    }
    
    pub fn as_bytes(&self) -> Result<&[u8]> {
        match self {
            QJSValue::ArrayBuffer(bytes) => Ok(bytes.as_slice()),
            _ => Err(anyhow!("Can't represent as an array buffer")),
        }
    }

    pub fn as_i32(&self) -> Result<i32> {
        match self {
            QJSValue::Int(i) => Ok(*i),
            _ => Err(anyhow!("Can't represent as i32")),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            QJSValue::Bool(b) => Ok(*b),
            _ => Err(anyhow!("Can't represent as a bool")),
        }
    }
}

impl TryFrom<QJSValue> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
        match value {
            QJSValue::ArrayBuffer(bytes) => Ok(bytes),
            _ => Err(anyhow!("Can't represent as an array buffer")),
        }
    }
}

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