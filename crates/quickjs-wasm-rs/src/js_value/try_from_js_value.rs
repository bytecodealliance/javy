use super::JSValue;
use anyhow::anyhow;
use std::collections::HashMap;

macro_rules! impl_try_from_jsvalue {
    ($($t:ty, $variant:ident),+ $(,)?) => {
        $(impl TryFrom<JSValue> for $t {
            type Error = anyhow::Error;

            fn try_from(value: JSValue) -> Result<Self, Self::Error> {
                match value {
                    JSValue::$variant(val) => Ok(val),
                    _ => Err(anyhow!("Error: could not convert JSValue to {}", std::any::type_name::<$t>())),
                }
            }
        })+
    };
}

impl_try_from_jsvalue!(
    bool, Bool,
    i32, Int,
    f64, Float,
    String, String,
    Vec<JSValue>, Array,
    HashMap<String, JSValue>, Object,
    Vec<u8>, ArrayBuffer,
);

impl TryFrom<JSValue> for usize {
    type Error = anyhow::Error;

    fn try_from(value: JSValue) -> Result<Self, Self::Error> {
        match value {
            JSValue::Int(val) => Ok(val as usize),
            _ => Err(anyhow!("Error: could not convert JSValue to usize")),
        }
    }
}
