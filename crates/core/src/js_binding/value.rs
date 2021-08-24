#![allow(dead_code)]
use super::context::Context;
use anyhow::{anyhow, Result};
use quickjs_sys::{
    JSValue, JS_DefinePropertyValueStr, JS_DefinePropertyValueUint32, JS_GetPropertyStr,
    JS_GetPropertyUint32, JS_IsArray, JS_IsFloat64_Ext, JS_NewArray, JS_NewBool_Ext,
    JS_NewFloat64_Ext, JS_NewInt32_Ext, JS_NewObject, JS_NewStringLen, JS_NewUint32_Ext,
    JS_ToFloat64, JS_PROP_C_W_E, JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_INT, JS_TAG_OBJECT,
    JS_TAG_STRING, JSContext
};
use std::{ffi::CString, os::raw::c_char};

#[derive(Debug, Clone)]
pub(crate) struct Value {
    context: *mut JSContext,
    value: JSValue,
}

pub trait PropertyAccess<K, V> {
    fn get(&self, key: K) -> Result<V>;
    fn set(&self, key: K, val: &V) -> Result<()>;
}

impl PropertyAccess<&str, Value> for Value {
    fn get(&self, key: &str) -> Result<Self> {
        let cstring_key = CString::new(key)?;
        let raw =
            unsafe { JS_GetPropertyStr(self.context, self.value, cstring_key.as_ptr()) };

        Self::new(self.context, raw)
    }

    fn set(&self, key: &str, val: &Value) -> Result<()> {
        let cstring_key = CString::new(key)?;
        // TODO: Exception handling
        let _raw = unsafe {
            JS_DefinePropertyValueStr(
                self.context,
                self.value,
                cstring_key.as_ptr(),
                val.value,
                JS_PROP_C_W_E as i32,
            )
        };

        Ok(())
    }
}

impl PropertyAccess<u32, Value> for Value {
    fn get(&self, key: u32) -> Result<Self> {
        let raw = unsafe { JS_GetPropertyUint32(self.context, self.value, key) };
        Self::new(self.context, raw)
    }

    fn set(&self, _key: u32, val: &Value) -> Result<()> {
        unsafe {
            let len = self.get("length")?;
            JS_DefinePropertyValueUint32(
                self.context,
                self.value,
                len.value as u32,
                val.value,
                JS_PROP_C_W_E as i32,
            );
        }
        Ok(())
    }
}

impl Value {
    pub(crate) fn new(context: *mut JSContext, raw_value: JSValue) -> Result<Self> {
        let tag = get_tag(raw_value);

        if is_exception(tag) {
            Err(anyhow!("Exception thrown by the JavaScript engine"))
        } else {
            Ok(Self {
                context,
                value: raw_value,
            })
        }
    }

    pub(crate) fn array(context: *mut JSContext) -> Result<Self> {
        let raw = unsafe { JS_NewArray(context) };
        Self::new(context, raw)
    }

    pub(crate) fn object(context: *mut JSContext) -> Result<Self> {
        let raw = unsafe { JS_NewObject(context) };
        Self::new(context, raw)
    }

    pub(crate) fn from_f64(context: *mut JSContext, val: f64) -> Result<Self> {
        let raw = unsafe { JS_NewFloat64_Ext(context, val) };
        Self::new(context, raw)
    }

    pub(crate) fn from_i32(context: *mut JSContext, val: i32) -> Result<Self> {
        let raw = unsafe { JS_NewInt32_Ext(context, val) };
        Self::new(context, raw)
    }

    pub(crate) fn from_u32(context: *mut JSContext, val: u32) -> Result<Self> {
        let raw = unsafe { JS_NewUint32_Ext(context, val) };
        Self::new(context, raw)
    }

    pub(crate) fn from_bool(context: *mut JSContext, val: bool) -> Result<Self> {
        let raw = unsafe { JS_NewBool_Ext(context, i32::from(val)) };
        Self::new(context, raw)
    }

    pub(crate) fn from_str(context: *mut JSContext, val: &str) -> Result<Self> {
        let raw = unsafe {
            JS_NewStringLen(
                context,
                val.as_ptr() as *const c_char,
                val.len() as _,
            )
        };
        Self::new(context, raw)
    }

    pub(crate) fn as_f64(&self) -> f64 {
        let mut ret = 0_f64;
        unsafe { JS_ToFloat64(self.context, &mut ret, self.value) };
        ret
    }

    pub(crate) fn inner(&self) -> JSValue {
        self.value
    }

    pub(crate) fn is_repr_as_f64(&self) -> bool {
        unsafe { JS_IsFloat64_Ext(get_tag(self.value)) == 1 }
    }

    pub(crate) fn is_repr_as_i32(&self) -> bool {
        matches!(get_tag(self.value), JS_TAG_INT)
    }

    pub(crate) fn is_str(&self) -> bool {
        matches!(get_tag(self.value), JS_TAG_STRING)
    }

    pub(crate) fn is_bool(&self) -> bool {
        matches!(get_tag(self.value), JS_TAG_BOOL)
    }

    pub(crate) fn is_array(&self) -> bool {
        unsafe { JS_IsArray(self.context, self.value) == 1 }
    }

    pub(crate) fn is_object(&self) -> bool {
        !self.is_array() && matches!(get_tag(self.value), JS_TAG_OBJECT)
    }
}

fn get_tag(v: JSValue) -> i32 {
    (v >> 32) as i32
}

fn is_exception(t: i32) -> bool {
    matches!(t, JS_TAG_EXCEPTION)
}

#[cfg(test)]
mod tests {
    use super::{Context, PropertyAccess, Value};
    use anyhow::Result;
    const SCRIPT_NAME: &str = "value.js";

    #[test]
    fn test_value_objects_allow_retrieving_a_str_property() -> Result<()> {
        let ctx = Context::new()?;
        let contents = "globalThis.bar = 1;";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let prop = global.get("bar");
        assert!(prop.is_ok());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_a_str_property() -> Result<()> {
        let ctx = Context::new()?;
        let obj = Value::object(ctx.inner())?;
        obj.set("foo", &Value::from_i32(ctx.inner(), 1_i32)?)?;
        let val = obj.get("foo");
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_a_indexed_property() -> Result<()> {
        let ctx = Context::new()?;
        let seq = Value::array(ctx.inner())?;
        seq.set(0_u32, &Value::from_str(ctx.inner(), "value")?)?;
        let val = seq.get(0);
        assert!(val.is_ok());
        assert!(val.unwrap().is_str());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_retrieving_a_indexed_property() -> Result<()> {
        let ctx = Context::new()?;
        let contents = "globalThis.arr = [1];";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let val = ctx.global_object()?.get("arr")?.get(0);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_f64() -> Result<()> {
        let ctx = Context::new()?;
        let val = f64::MIN;
        let val = Value::from_f64(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_f64());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_i32() -> Result<()> {
        let ctx = Context::new()?;
        let val = i32::MIN;
        let val = Value::from_i32(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_u32() -> Result<()> {
        let ctx = Context::new()?;
        let val = u32::MIN;
        let val = Value::from_u32(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_bool() -> Result<()> {
        let ctx = Context::new()?;
        let val = false;
        let val = Value::from_bool(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_bool());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_str() -> Result<()> {
        let ctx = Context::new()?;
        let val = "script.js";
        let val = Value::from_str(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_str());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_array() -> Result<()> {
        let ctx = Context::new()?;
        let val = Value::array(ctx.inner());
        assert!(val.is_ok());
        assert!(val.unwrap().is_array());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_object() -> Result<()> {
        let ctx = Context::new()?;
        let val = Value::object(ctx.inner());
        assert!(val.is_ok());
        assert!(val.unwrap().is_object());
        Ok(())
    }

    #[test]
    fn test_allows_representing_a_value_as_f64() -> Result<()> {
        let ctx = Context::new()?;
        let val = Value::from_f64(ctx.inner(), f64::MIN)?.as_f64();
        assert_eq!(val, f64::MIN);
        Ok(())
    }
}
