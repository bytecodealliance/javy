use std::{fmt, convert::TryInto, collections::HashMap};

use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq)]
pub enum JSValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Array(Vec<JSValue>),
    ArrayBuffer(Vec<u8>),
    Object(HashMap<String, JSValue>),
}

impl TryInto<bool> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<bool> {
        match self {
            JSValue::Bool(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to bool")),
        }
    }
}

impl TryInto<i32> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<i32> {
        match self {
            JSValue::Int(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to i32")),
        }
    }
}

impl TryInto<usize> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<usize> {
        match self {
            JSValue::Int(val) => Ok(val as usize),
            JSValue::Float(val) => Ok(val.floor() as usize),
            _ => Err(anyhow!("Error: could not convert JSValue to usize")),
        }
    }
}

impl TryInto<f64> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<f64> {
        match self {
            JSValue::Float(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to f64")),
        }
    }
}

impl TryInto<String> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String> {
        match self {
            JSValue::String(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to String")),
        }
    }
}

impl TryInto<Vec<JSValue>> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<JSValue>> {
        match self {
            JSValue::Array(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to Vec<JSValue>")),
        }
    }
}

impl TryInto<HashMap<String, JSValue>> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<HashMap<String, JSValue>> {
        match self {
            JSValue::Object(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to HashMap<String, JSValue>")),
        }
    }
}

impl TryInto<Vec<u8>> for JSValue {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        match self {
            JSValue::ArrayBuffer(val) => Ok(val),
            _ => Err(anyhow!("Error: could not convert JSValue to Vec<u8>")),
        }
    }
}

// Used http://numcalc.com/ to playaround and determine the default display format for each type
impl fmt::Display for JSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSValue::Undefined => write!(f, "undefined"),
            JSValue::Null => write!(f, "null"),
            JSValue::Bool(b) => write!(f, "{}", b),
            JSValue::Int(i) => write!(f, "{}", i),
            JSValue::Float(n) => write!(f, "{}", n),
            JSValue::String(s) => write!(f, "{}", s),
            // JSValue::MutArrayBuffer(_, _) => write!(f, "ArrayBuffer"),
            JSValue::ArrayBuffer(buffer) => write!(f, "{:?}", buffer),
            JSValue::Array(arr) => {
                write!(f, "{}", arr.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(","))     
            }
            JSValue::Object(_) => write!(f, "[object Object]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_value_try_into_bool() {
        let js_value = JSValue::Bool(true);
        assert_eq!("true", js_value.to_string());
        
        let result: bool = js_value.try_into().unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_js_value_try_into_f64() {   
        let js_value = JSValue::Float(2.3);
        assert_eq!("2.3", js_value.to_string());
        
        let result: f64 = js_value.try_into().unwrap();
        assert_eq!(result, 2.3);
    }
}