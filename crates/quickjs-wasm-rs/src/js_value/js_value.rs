use std::collections::HashMap;

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
    Object(HashMap<String, JSValue>),
}

impl JSValue {
    pub fn as_i32(&self) -> Result<i32> {
        match self {
            JSValue::Int(i) => Ok(*i),
            _ => Err(anyhow!("expected Int")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_i32_success() {
        let v = JSValue::Int(42);
        assert_eq!(v.as_i32().unwrap(), 42);
    }

    #[test]
    fn test_as_i32_failure() {
        let v = JSValue::Float(42.0);
        assert!(v.as_i32().is_err());
    }
}