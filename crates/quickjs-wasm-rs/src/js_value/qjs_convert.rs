use std::collections::HashMap;

use anyhow::{bail, Result};
use quickjs_wasm_sys::{
    JS_TAG_BOOL, JS_TAG_INT, JS_TAG_NULL, JS_TAG_OBJECT, JS_TAG_STRING, JS_TAG_UNDEFINED,
};

use super::JSValue;
use crate::js_binding::{context::JSContextRef, value::JSValueRef};

/// Converts a reference to QuickJS value represented by `quickjs_wasm_rs::JSValueRef` to a `JSValue`.
///
/// # Arguments
///
/// * `val` - a `JSValueRef` to be converted to a `JSValue`
///
/// # Returns
///
/// * `anyhow::Result<JSValue>`
///
/// # Example
///
/// ```
/// // Assuming `qjs_val` is a `JSValueRef` pointing to a QuickJS string
/// let js_val = from_qjs_value(qjs_val).unwrap();
/// assert_eq!(js_val, "hello".into());
/// ```
pub fn from_qjs_value(val: JSValueRef) -> Result<JSValue> {
    let tag = val.get_tag();
    let js_val = match tag {
        JS_TAG_NULL => JSValue::Null,
        JS_TAG_UNDEFINED => JSValue::Undefined,
        JS_TAG_BOOL => JSValue::Bool(val.as_bool()?),
        JS_TAG_INT => JSValue::Int(val.as_i32_unchecked()),
        JS_TAG_STRING => {
            // need to use as_str_lossy here otherwise a wpt test fails because there is a test case
            // that has a string with invalid utf8
            JSValue::String(val.as_str_lossy().to_string())
        }
        JS_TAG_OBJECT => {
            if val.is_array() {
                let array_len = from_qjs_value(val.get_property("length")?)?.try_into()?;
                let mut result = Vec::with_capacity(array_len);
                for i in 0..array_len {
                    result.push(from_qjs_value(val.get_indexed_property(i.try_into()?)?)?);
                }
                JSValue::Array(result)
            } else if val.is_array_buffer() {
                let bytes = val.as_bytes()?;
                JSValue::ArrayBuffer(bytes.to_vec())
            } else {
                let mut result = HashMap::new();
                let mut properties = val.properties()?;
                while let Some(property_key) = properties.next_key()? {
                    let property_key = property_key.as_str()?;
                    let property_value = from_qjs_value(val.get_property(property_key)?)?;
                    result.insert(property_key.to_string(), property_value);
                }

                JSValue::Object(result)
            }
        }
        _ if val.is_repr_as_f64() => {
            // Matching on JS_TAG_FLOAT64 does not seem to catch floats so we have to check for float separately.
            JSValue::Float(val.as_f64_unchecked())
        }
        _ => bail!("unhandled tag: {}", tag),
    };
    Ok(js_val)
}

/// Converts a reference to a `JSValue` to a `quickjs_wasm_rs::JSValueRef`.
///
/// # Arguments
///
/// * `context` - a reference to a `quickjs_wasm_rs::JSContextRef`. The `JSValueRef` is created for this JS context.
/// * `val` - a reference to a `JSValue` to be converted to a `JSValueRef`
///
/// # Returns
///
/// * `anyhow::Result<JSValueRef>`
///
/// # Example
///
/// ```
/// let context = JSContextRef::default();
/// let js_val = "hello".into();
/// let qjs_val = to_qjs_value(&context, &js_val).unwrap();
/// ```
pub fn to_qjs_value<'a>(context: &'a JSContextRef, val: &JSValue) -> Result<JSValueRef<'a>> {
    let qjs_val = match val {
        JSValue::Undefined => context.undefined_value()?,
        JSValue::Null => context.null_value()?,
        JSValue::Bool(flag) => context.value_from_bool(*flag)?,
        JSValue::Int(val) => context.value_from_i32(*val)?,
        JSValue::Float(val) => context.value_from_f64(*val)?,
        JSValue::String(val) => context.value_from_str(val)?,
        JSValue::ArrayBuffer(buffer) => context.array_buffer_value(buffer)?,
        JSValue::Array(values) => {
            let arr = context.array_value()?;
            for v in values.iter() {
                arr.append_property(to_qjs_value(context, v)?)?
            }
            arr
        }
        JSValue::Object(hashmap) => {
            let obj = context.object_value()?;
            for (key, value) in hashmap {
                obj.set_property(key.clone(), to_qjs_value(context, value)?)?
            }
            obj
        }
    };
    Ok(qjs_val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_qjs_null() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "null").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, JSValue::Null);
    }

    #[test]
    fn test_from_qjs_undefined() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "undefined").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, JSValue::Undefined);
    }

    #[test]
    fn test_from_qjs_bool() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "true").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, true.into());
    }

    #[test]
    fn test_from_qjs_int() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "42").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, 42.into());
    }

    #[test]
    fn test_from_qjs_float() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "42.5").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, 42.5.into());
    }

    #[test]
    fn test_from_qjs_string() {
        let context = JSContextRef::default();
        let qjs_val = context
            .eval_global("test.js", "const h = 'hello'; h")
            .unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, "hello".into());
    }

    #[test]
    fn test_from_qjs_array() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "[1, 2, 3]").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(
            js_val,
            vec![JSValue::Int(1), JSValue::Int(2), JSValue::Int(3)].into()
        );
    }

    #[test]
    fn test_from_qjs_array_buffer() {
        let context = JSContextRef::default();
        let qjs_val = context
            .eval_global("test.js", "new ArrayBuffer(8)")
            .unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(js_val, [0_u8; 8].as_ref().into());
    }

    #[test]
    fn test_from_qjs_object() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "({a: 1, b: 2})").unwrap();
        let js_val = from_qjs_value(qjs_val).unwrap();
        assert_eq!(
            js_val,
            HashMap::from([
                ("a".to_string(), JSValue::Int(1)),
                ("b".to_string(), JSValue::Int(2))
            ])
            .into()
        )
    }

    #[test]
    fn test_to_qjs_null() {
        let context = JSContextRef::default();
        let js_val = JSValue::Null;
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert!(qjs_val.is_null());
    }

    #[test]
    fn test_to_qjs_undefined() {
        let context = JSContextRef::default();
        let js_val = JSValue::Undefined;
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert!(qjs_val.is_undefined());
    }

    #[test]
    fn test_to_qjs_bool() {
        let context = JSContextRef::default();
        let js_val = true.into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert!(qjs_val.as_bool().unwrap());
    }

    #[test]
    fn test_to_qjs_int() {
        let context = JSContextRef::default();
        let js_val = 42.into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(42, qjs_val.as_i32_unchecked());
    }

    #[test]
    fn test_to_qjs_float() {
        let context = JSContextRef::default();
        let js_val = 42.3.into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(42.3, qjs_val.as_f64_unchecked());
    }

    #[test]
    fn test_to_qjs_string() {
        let context = JSContextRef::default();
        let js_val = "hello".into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!("hello", qjs_val.as_str().unwrap());
    }

    #[test]
    fn test_to_qjs_array_buffer() {
        let context = JSContextRef::default();
        let js_val = "hello".as_bytes().into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!("hello".as_bytes(), qjs_val.as_bytes().unwrap());
    }

    #[test]
    fn test_to_qjs_array() {
        let context = JSContextRef::default();
        let js_val = vec![JSValue::Int(1), JSValue::Int(2)].into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(1, qjs_val.get_property("0").unwrap().as_i32_unchecked());
        assert_eq!(2, qjs_val.get_property("1").unwrap().as_i32_unchecked());
    }

    #[test]
    fn test_to_qjs_object() {
        let context = JSContextRef::default();
        let js_val = HashMap::from([
            ("a".to_string(), JSValue::Int(1)),
            ("b".to_string(), JSValue::Int(2)),
        ])
        .into();
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(1, qjs_val.get_property("a").unwrap().as_i32_unchecked());
        assert_eq!(2, qjs_val.get_property("b").unwrap().as_i32_unchecked());
    }
}
