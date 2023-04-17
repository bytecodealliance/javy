use super::JSValue;
use std::collections::HashMap;
macro_rules! impl_to_jsvalue {
    ($($t:ty, $variant:ident),+ $(,)?) => {
        $(impl From<$t> for JSValue {
            fn from(value: $t) -> Self {
                JSValue::$variant(value)
            }
        })+
    };
}

impl_to_jsvalue!(
    bool, Bool,
    i32, Int,
    f64, Float,
    String, String,
    Vec<JSValue>, Array,
    Vec<u8>, ArrayBuffer,
    HashMap<String, JSValue>, Object,
);

impl From<usize> for JSValue {
    fn from(value: usize) -> Self {
        JSValue::Int(value as i32)
    }
}

impl From<&str> for JSValue {
    fn from(value: &str) -> Self {
        JSValue::String(value.to_string())
    }
}

impl From<&[u8]> for JSValue {
    fn from(value: &[u8]) -> Self {
        JSValue::ArrayBuffer(value.to_vec())
    }
}
