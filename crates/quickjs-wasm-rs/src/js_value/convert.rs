use std::collections::HashMap;

use anyhow::Result;
use quickjs_wasm_sys::{
    JS_TAG_BOOL, JS_TAG_INT, JS_TAG_NULL, JS_TAG_OBJECT, JS_TAG_STRING, JS_TAG_UNDEFINED,
};

use super::js_value::JSValue;
use crate::js_binding::{context::JSContextRef, value::JSValueRef};

pub fn from_qjs_value(context: &JSContextRef, val: &JSValueRef) -> Result<JSValue> {
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
                let array_len =
                    from_qjs_value(context, &val.get_property("length")?)?.try_into()?;
                let mut result = Vec::new();
                for i in 0..array_len {
                    result.push(from_qjs_value(
                        context,
                        &val.get_indexed_property(i.try_into()?)?,
                    )?);
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
                    let property_value = from_qjs_value(context, &val.get_property(property_key)?)?;
                    result.insert(property_key.to_string(), property_value);
                }

                JSValue::Object(result)
            }
        }
        _ => {
            // Matching on JS_TAG_FLOAT64 does not seem to catch floats so we have to check for float separately
            if val.is_repr_as_f64() {
                JSValue::Float(val.as_f64_unchecked())
            } else {
                panic!("unhandled tag: {}", tag)
            }
        }
    };
    Ok(js_val)
}

pub fn to_qjs_value(context: &JSContextRef, val: &JSValue) -> Result<JSValueRef> {
    let qjs_val = match val {
        JSValue::Undefined => context.undefined_value()?,
        JSValue::Null => context.null_value()?,
        JSValue::Bool(flag) => context.value_from_bool(*flag)?,
        JSValue::Int(val) => context.value_from_i32(*val)?,
        JSValue::Float(val) => context.value_from_f64(*val)?,
        JSValue::String(val) => context.value_from_str(&val)?,
        JSValue::ArrayBuffer(buffer) => context.array_buffer_value(&buffer)?,
        JSValue::Array(values) => {
            let arr = context.array_value()?;
            for v in values.into_iter() {
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
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(js_val, JSValue::Null);
    }

    #[test]
    fn test_from_qjs_undefined() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "undefined").unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(js_val, JSValue::Undefined);
    }

    #[test]
    fn test_from_qjs_bool() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "true").unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(js_val, JSValue::Bool(true));
    }

    #[test]
    fn test_from_qjs_int() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "42").unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(js_val, JSValue::Int(42));
    }

    #[test]
    fn test_from_qjs_string() {
        let context = JSContextRef::default();
        let qjs_val = context
            .eval_global("test.js", "const h = 'hello'; h")
            .unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(js_val, JSValue::String("hello".to_string()));
    }

    #[test]
    fn test_from_qjs_array() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "[1, 2, 3]").unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(
            js_val,
            JSValue::Array(vec![JSValue::Int(1), JSValue::Int(2), JSValue::Int(3)])
        );
    }

    #[test]
    fn test_from_qjs_array_buffer() {
        let context = JSContextRef::default();
        let qjs_val = context
            .eval_global("test.js", "new ArrayBuffer(8)")
            .unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(js_val, JSValue::ArrayBuffer(vec![0, 0, 0, 0, 0, 0, 0, 0]));
    }

    #[test]
    fn test_from_qjs_object() {
        let context = JSContextRef::default();
        let qjs_val = context.eval_global("test.js", "({a: 1, b: 2})").unwrap();
        let js_val = from_qjs_value(&context, &qjs_val).unwrap();
        assert_eq!(
            js_val,
            JSValue::Object(HashMap::from_iter(vec![
                ("a".to_string(), JSValue::Int(1)),
                ("b".to_string(), JSValue::Int(2))
            ]))
        )
    }

    #[test]
    fn test_to_qjs_null() {
        let context = JSContextRef::default();
        let js_val = JSValue::Null;
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(qjs_val.is_null(), true);
    }

    #[test]
    fn test_to_qjs_undefined() {
        let context = JSContextRef::default();
        let js_val = JSValue::Undefined;
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(qjs_val.is_undefined(), true);
    }

    #[test]
    fn test_to_qjs_bool() {
        let context = JSContextRef::default();
        let js_val = JSValue::Bool(true);
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(qjs_val.as_bool().unwrap(), true);
    }

    #[test]
    fn test_to_qjs_int() {
        let context = JSContextRef::default();
        let js_val = JSValue::Int(42);
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(42, qjs_val.as_i32_unchecked());
    }

    #[test]
    fn test_to_qjs_float() {
        let context = JSContextRef::default();
        let js_val = JSValue::Float(42.0);
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(42.0, qjs_val.as_f64_unchecked());
    }

    #[test]
    fn test_to_qjs_string() {
        let context = JSContextRef::default();
        let js_val = JSValue::String("hello".to_string());
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!("hello", qjs_val.as_str().unwrap());
    }

    #[test]
    fn test_to_qjs_array_buffer() {
        let context = JSContextRef::default();
        let js_val = JSValue::ArrayBuffer(vec![0, 1]);
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!([0, 1], qjs_val.as_bytes().unwrap());
    }

    #[test]
    fn test_to_qjs_array() {
        let context = JSContextRef::default();
        let js_val = JSValue::Array(vec![JSValue::Int(1), JSValue::Int(2)]);
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(1, qjs_val.get_property("0").unwrap().as_i32_unchecked());
        assert_eq!(2, qjs_val.get_property("1").unwrap().as_i32_unchecked());
    }

    #[test]
    fn test_to_qjs_object() {
        let context = JSContextRef::default();
        let js_val = JSValue::Object(HashMap::from_iter(vec![
            ("a".to_string(), JSValue::Int(1)),
            ("b".to_string(), JSValue::Int(2)),
        ]));
        let qjs_val = to_qjs_value(&context, &js_val).unwrap();
        assert_eq!(1, qjs_val.get_property("a").unwrap().as_i32_unchecked());
        assert_eq!(2, qjs_val.get_property("b").unwrap().as_i32_unchecked());
    }
}
