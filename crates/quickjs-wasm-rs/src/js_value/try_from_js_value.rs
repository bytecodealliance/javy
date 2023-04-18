use super::JSValue;
use anyhow::anyhow;
use std::collections::HashMap;

/// A macro for implementing `TryFrom<JSValue>` for multiple types at once.
/// Takes a list of type-variant pairs and generates a `TryFrom<JSValue>` implementation for each type.
///
/// # Type-Variant Pairs
///
/// * `$t:ty` - The type for which the implementation is generated
/// * `$variant:ident` - The corresponding variant of `JSValue` that is expected when converting
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
