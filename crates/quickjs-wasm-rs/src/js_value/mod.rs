pub mod convert;

use std::{collections::HashMap, convert::TryInto, fmt};

use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq, Clone)]
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

macro_rules! jsvalue_try_into_impl {
    ($($t:ty, $variant:ident, $conv:expr),+ $(,)?) => {
        $(impl TryInto<$t> for JSValue {
            type Error = anyhow::Error;

            fn try_into(self) -> Result<$t> {
                match self {
                    JSValue::$variant(val) => Ok($conv(val)),
                    _ => Err(anyhow!("Error: could not convert JSValue to {}", std::any::type_name::<$t>())),
                }
            }
        })+
    };
}

jsvalue_try_into_impl!(
    bool, Bool, |x| x,
    i32, Int, |x| x,
    usize, Int, |x| x as usize,
    f64, Float, |x| x,
    String, String, |x| x,
    Vec<JSValue>, Array, |x| x,
    HashMap<String, JSValue>, Object, |x| x,
    Vec<u8>, ArrayBuffer, |x| x,
);

// Used http://numcalc.com/ to determine the default display format for each type
impl fmt::Display for JSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSValue::Undefined => write!(f, "undefined"),
            JSValue::Null => write!(f, "null"),
            JSValue::Bool(b) => write!(f, "{}", b),
            JSValue::Int(i) => write!(f, "{}", i),
            JSValue::Float(n) => write!(f, "{}", n),
            JSValue::String(s) => write!(f, "{}", s),
            JSValue::ArrayBuffer(_) => write!(f, "{{  }}"),
            JSValue::Array(arr) => {
                write!(
                    f,
                    "{}",
                    arr.iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>()
                        .join(",")
                )
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
        assert!(result);
    }

    #[test]
    fn test_js_value_try_into_i32() {
        let js_value = JSValue::Int(2);
        assert_eq!("2", js_value.to_string());

        let result: i32 = js_value.try_into().unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_js_value_try_into_f64() {
        let js_value = JSValue::Float(2.3);
        assert_eq!("2.3", js_value.to_string());

        let result: f64 = js_value.try_into().unwrap();
        assert_eq!(result, 2.3);
    }

    #[test]
    fn test_js_value_try_into_string() {
        let js_value = JSValue::String("hello".to_string());
        assert_eq!("hello", js_value.to_string());

        let result: String = js_value.try_into().unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_js_value_try_into_array() {
        let js_value = JSValue::Array(vec![JSValue::Int(1), JSValue::Int(2)]);
        assert_eq!("1,2", js_value.to_string());

        let result: Vec<JSValue> = js_value.try_into().unwrap();
        assert_eq!(result, vec![JSValue::Int(1), JSValue::Int(2)]);
    }

    #[test]
    fn test_js_value_try_into_array_buffer() {
        let js_value = JSValue::ArrayBuffer(vec![1, 2, 3]);
        assert_eq!("{  }", js_value.to_string());

        let result: Vec<u8> = js_value.try_into().unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_js_value_try_into_object() {
        let mut obj = HashMap::new();
        obj.insert("a".to_string(), JSValue::Int(1));
        obj.insert("b".to_string(), JSValue::Int(2));
        let js_value = JSValue::Object(obj.clone());
        assert_eq!("[object Object]", js_value.to_string());

        let result: HashMap<String, JSValue> = js_value.try_into().unwrap();
        assert_eq!(result, obj);
    }
}
