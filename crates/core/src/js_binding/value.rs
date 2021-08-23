use super::context::Context;
use anyhow::{anyhow, Result};
use quickjs_sys::{
    JSValue, JS_GetPropertyStr, JS_IsArray, JS_IsFloat64_Ext, JS_NewArray, JS_NewBool_Ext,
    JS_NewFloat64_Ext, JS_NewInt32_Ext, JS_NewObject, JS_NewStringLen, JS_NewUint32_Ext,
    JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_INT, JS_TAG_OBJECT, JS_TAG_STRING,
};
use std::{ffi::CString, os::raw::c_char};

#[derive(Debug, Clone)]
pub(crate) struct Value<'v> {
    context: &'v Context,
    value: JSValue,
    tag: i32,
}

impl<'v> Value<'v> {
    pub(crate) fn new(context: &'v Context, raw_value: JSValue) -> Result<Self> {
        let tag = get_tag(raw_value);

        if is_exception(tag) {
            Err(anyhow!("Exception thrown by the JavaScript engine"))
        } else {
            Ok(Self {
                context,
                value: raw_value,
                tag,
            })
        }
    }

    pub(crate) fn array(context: &'v Context) -> Result<Self> {
        let raw = unsafe { JS_NewArray(context.inner()) };
        Self::new(context, raw)
    }

    pub(crate) fn object(context: &'v Context) -> Result<Self> {
        let raw = unsafe { JS_NewObject(context.inner()) };
        Self::new(context, raw)
    }

    pub(crate) fn from_f64(context: &'v Context, val: f64) -> Result<Self> {
        let raw = unsafe { JS_NewFloat64_Ext(context.inner(), val) };
        Self::new(context, raw)
    }

    pub(crate) fn from_i32(context: &'v Context, val: i32) -> Result<Self> {
        let raw = unsafe { JS_NewInt32_Ext(context.inner(), val) };
        Self::new(context, raw)
    }

    pub(crate) fn from_u32(context: &'v Context, val: u32) -> Result<Self> {
        let raw = unsafe { JS_NewUint32_Ext(context.inner(), val) };
        Self::new(context, raw)
    }

    pub(crate) fn from_bool(context: &'v Context, val: bool) -> Result<Self> {
        let raw = unsafe { JS_NewBool_Ext(context.inner(), i32::from(val)) };
        Self::new(context, raw)
    }

    pub(crate) fn from_str(context: &'v Context, val: &str) -> Result<Self> {
        let raw = unsafe {
            JS_NewStringLen(
                context.inner(),
                val.as_ptr() as *const c_char,
                val.len() as _,
            )
        };
        Self::new(context, raw)
    }

    pub(crate) fn inner(&self) -> JSValue {
        self.value
    }

    pub(crate) fn property(&self, key: &str) -> Result<Self> {
        let cstring_key = CString::new(key)?;
        let raw =
            unsafe { JS_GetPropertyStr(self.context.inner(), self.value, cstring_key.as_ptr()) };

        Self::new(self.context, raw)
    }

    pub(crate) fn is_repr_as_f64(&self) -> bool {
        unsafe { JS_IsFloat64_Ext(get_tag(self.value)) == 1 }
    }

    pub(crate) fn is_repr_as_i32(&self) -> bool {
        get_tag(self.value) == JS_TAG_INT
    }

    pub(crate) fn is_string(&self) -> bool {
        get_tag(self.value) == JS_TAG_STRING
    }

    pub(crate) fn is_bool(&self) -> bool {
        get_tag(self.value) == JS_TAG_BOOL
    }

    pub(crate) fn is_array(&self) -> bool {
        unsafe { JS_IsArray(self.context.inner(), self.value) == 1 }
    }

    pub(crate) fn is_object(&self) -> bool {
        !self.is_array() && get_tag(self.value) == JS_TAG_OBJECT
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
    use super::{Context, Value};
    use anyhow::Result as R;
    const SCRIPT_NAME: &str = "value.js";

    #[test]
    fn test_value_objects_allow_retrieving_a_property() -> R<()> {
        let ctx = Context::new()?;
        let contents = "globalThis.bar = 1;";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let prop = global.property("bar");
        assert!(prop.is_ok());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_f64() -> R<()> {
        let ctx = Context::new()?;
        let val = f64::MIN;
        let val = Value::from_f64(&ctx, val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_f64());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_i32() -> R<()> {
        let ctx = Context::new()?;
        let val = i32::MIN;
        let val = Value::from_i32(&ctx, val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_u32() -> R<()> {
        let ctx = Context::new()?;
        let val = u32::MIN;
        let val = Value::from_u32(&ctx, val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_bool() -> R<()> {
        let ctx = Context::new()?;
        let val = false;
        let val = Value::from_bool(&ctx, val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_bool());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_str() -> R<()> {
        let ctx = Context::new()?;
        let val = "script.js";
        let val = Value::from_str(&ctx, val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_string());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_array() -> R<()> {
        let ctx = Context::new()?;
        let val = Value::array(&ctx);
        assert!(val.is_ok());
        assert!(val.unwrap().is_array());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_object() -> R<()> {
        let ctx = Context::new()?;
        let val = Value::object(&ctx);
        assert!(val.is_ok());
        assert!(val.unwrap().is_object());
        Ok(())
    }
}
