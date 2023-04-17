use super::JSValue;
use std::collections::HashMap;

/// A macro for implementing `From<T>` for `JSValue` for multiple types at once.
/// Takes a list of type-variant pairs and generates a `From<T>` implementation for `JSValue` for each type.
///
/// # Type-Variant Pairs
///
/// * `$t:ty` - The type from which the conversion is done
/// * `$variant:ident` - The corresponding variant of `JSValue` that will be created when converting
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
