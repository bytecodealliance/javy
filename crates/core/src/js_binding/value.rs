use super::own_properties::OwnProperties;
use anyhow::{anyhow, Result};
use quickjs_sys::{
    ext_js_null, ext_js_undefined, size_t as JS_size_t, JSContext, JSValue,
    JS_DefinePropertyValueStr, JS_DefinePropertyValueUint32, JS_GetException, JS_GetPropertyStr,
    JS_GetPropertyUint32, JS_IsArray, JS_IsError, JS_IsFloat64_Ext, JS_NewArray, JS_NewBool_Ext,
    JS_NewFloat64_Ext, JS_NewInt32_Ext, JS_NewObject, JS_NewStringLen, JS_NewUint32_Ext,
    JS_ToCStringLen2, JS_ToFloat64, JS_PROP_C_W_E, JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_INT,
    JS_TAG_NULL, JS_TAG_OBJECT, JS_TAG_STRING, JS_TAG_UNDEFINED,
};
use std::fmt;
use std::{ffi::CString, os::raw::c_char};

#[derive(Debug, Clone)]
pub struct Value {
    context: *mut JSContext,
    value: JSValue,
}

#[derive(Debug)]
struct Exception {
    msg: String,
    stack: Option<String>,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        if let Some(stack) = &self.stack {
            write!(f, "\n{}", stack)?;
        }
        Ok(())
    }
}

impl Value {
    pub fn new(context: *mut JSContext, raw_value: JSValue) -> Result<Self> {
        let value = Self {
            context,
            value: raw_value,
        };

        if value.is_exception() {
            let exception = value.as_exception()?;
            Err(anyhow!("Uncaught {}", exception))
        } else {
            Ok(value)
        }
    }

    pub fn array(context: *mut JSContext) -> Result<Self> {
        let raw = unsafe { JS_NewArray(context) };
        Self::new(context, raw)
    }

    pub fn object(context: *mut JSContext) -> Result<Self> {
        let raw = unsafe { JS_NewObject(context) };
        Self::new(context, raw)
    }

    pub fn from_f64(context: *mut JSContext, val: f64) -> Result<Self> {
        let raw = unsafe { JS_NewFloat64_Ext(context, val) };
        Self::new(context, raw)
    }

    pub fn from_i32(context: *mut JSContext, val: i32) -> Result<Self> {
        let raw = unsafe { JS_NewInt32_Ext(context, val) };
        Self::new(context, raw)
    }

    pub fn from_u32(context: *mut JSContext, val: u32) -> Result<Self> {
        let raw = unsafe { JS_NewUint32_Ext(context, val) };
        Self::new(context, raw)
    }

    pub fn from_bool(context: *mut JSContext, val: bool) -> Result<Self> {
        let raw = unsafe { JS_NewBool_Ext(context, i32::from(val)) };
        Self::new(context, raw)
    }

    pub fn from_str(context: *mut JSContext, val: &str) -> Result<Self> {
        let raw =
            unsafe { JS_NewStringLen(context, val.as_ptr() as *const c_char, val.len() as _) };
        Self::new(context, raw)
    }

    pub fn null(context: *mut JSContext) -> Result<Self> {
        Self::new(context, unsafe { ext_js_null })
    }

    pub fn undefined(context: *mut JSContext) -> Result<Self> {
        Self::new(context, unsafe { ext_js_undefined })
    }

    pub fn as_f64(&self) -> Result<f64> {
        if self.is_repr_as_f64() || self.is_repr_as_i32() {
            let mut ret = 0_f64;
            unsafe { JS_ToFloat64(self.context, &mut ret, self.value) };
            Ok(ret)
        } else {
            Err(anyhow!("Can't represent {:?} as f64", self.value))
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        if self.is_bool() {
            Ok(self.value as i32 > 0)
        } else {
            Err(anyhow!("Can't represent {:?} as bool", self.value))
        }
    }

    pub fn as_str(&self) -> Result<&str> {
        unsafe {
            let mut len: JS_size_t = 0;
            let ptr = JS_ToCStringLen2(self.context, &mut len, self.value, 0);
            let ptr = ptr as *const u8;
            let len = len as usize;
            let buffer = std::slice::from_raw_parts(ptr, len);
            std::str::from_utf8(buffer).map_err(Into::into)
        }
    }

    pub fn inner(&self) -> JSValue {
        self.value
    }

    pub fn inner_context(&self) -> *mut JSContext {
        self.context
    }

    pub fn own_properties(&self) -> Result<OwnProperties> {
        OwnProperties::from(self)
    }

    pub fn is_repr_as_f64(&self) -> bool {
        unsafe { JS_IsFloat64_Ext(self.get_tag()) == 1 }
    }

    pub fn is_repr_as_i32(&self) -> bool {
        self.get_tag() == JS_TAG_INT
    }

    pub fn is_str(&self) -> bool {
        self.get_tag() == JS_TAG_STRING
    }

    pub fn is_bool(&self) -> bool {
        self.get_tag() == JS_TAG_BOOL
    }

    pub fn is_array(&self) -> bool {
        unsafe { JS_IsArray(self.context, self.value) == 1 }
    }

    pub fn is_object(&self) -> bool {
        !self.is_array() && self.get_tag() == JS_TAG_OBJECT
    }

    pub fn is_undefined(&self) -> bool {
        self.get_tag() == JS_TAG_UNDEFINED
    }

    pub fn is_null(&self) -> bool {
        self.get_tag() == JS_TAG_NULL
    }

    pub fn get_property(&self, key: impl Into<Vec<u8>>) -> Result<Self> {
        let cstring_key = CString::new(key)?;
        let raw = unsafe { JS_GetPropertyStr(self.context, self.value, cstring_key.as_ptr()) };
        Self::new(self.context, raw)
    }

    pub fn set_property(&self, key: impl Into<Vec<u8>>, val: &Value) -> Result<()> {
        let cstring_key = CString::new(key)?;
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

    pub fn get_indexed_property(&self, index: u32) -> Result<Self> {
        let raw = unsafe { JS_GetPropertyUint32(self.context, self.value, index) };
        Self::new(self.context, raw)
    }

    pub fn append_property(&self, val: &Value) -> Result<()> {
        let len = self.get_property("length")?;
        unsafe {
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

    pub fn is_exception(&self) -> bool {
        self.get_tag() == JS_TAG_EXCEPTION
    }

    fn get_tag(&self) -> i32 {
        (self.value >> 32) as i32
    }

    /// All methods in quickjs return an exception value, not an object.
    /// To actually retrieve the exception, we need to retrieve the exception object from the global state.
    fn as_exception(&self) -> Result<Exception> {
        let exception_value = unsafe { JS_GetException(self.context) };
        let exception_obj = Self {
            context: self.context,
            value: exception_value,
        };

        let msg = exception_obj.as_str().map(ToString::to_string)?;
        let mut stack = None;

        let is_error = unsafe { JS_IsError(self.context, exception_obj.value) } != 0;
        if is_error {
            let stack_value = exception_obj.get_property("stack")?;
            if !stack_value.is_undefined() {
                stack.replace(stack_value.as_str().map(ToString::to_string)?);
            }
        }

        Ok(Exception { msg, stack })
    }
}

#[cfg(test)]
mod tests {
    use super::super::context::Context;
    use super::Value;
    use anyhow::Result;
    const SCRIPT_NAME: &str = "value.js";

    #[test]
    fn test_value_objects_allow_retrieving_a_str_property() -> Result<()> {
        let ctx = Context::default();
        let contents = "globalThis.bar = 1;";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let prop = global.get_property("bar");
        assert!(prop.is_ok());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_a_str_property() -> Result<()> {
        let ctx = Context::default();
        let obj = Value::object(ctx.inner())?;
        obj.set_property("foo", &Value::from_i32(ctx.inner(), 1_i32)?)?;
        let val = obj.get_property("foo");
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_an_indexed_property() -> Result<()> {
        let ctx = Context::default();
        let seq = Value::array(ctx.inner())?;
        seq.append_property(&Value::from_str(ctx.inner(), "value")?)?;
        let val = seq.get_indexed_property(0);
        assert!(val.is_ok());
        assert!(val.unwrap().is_str());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_retrieving_a_indexed_property() -> Result<()> {
        let ctx = Context::default();
        let contents = "globalThis.arr = [1];";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let val = ctx
            .global_object()?
            .get_property("arr")?
            .get_indexed_property(0);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_f64() -> Result<()> {
        let ctx = Context::default();
        let val = f64::MIN;
        let val = Value::from_f64(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_f64());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_i32() -> Result<()> {
        let ctx = Context::default();
        let val = i32::MIN;
        let val = Value::from_i32(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_u32() -> Result<()> {
        let ctx = Context::default();
        let val = u32::MIN;
        let val = Value::from_u32(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_bool() -> Result<()> {
        let ctx = Context::default();
        let val = false;
        let val = Value::from_bool(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_bool());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_str() -> Result<()> {
        let ctx = Context::default();
        let val = "script.js";
        let val = Value::from_str(ctx.inner(), val);
        assert!(val.is_ok());
        assert!(val.unwrap().is_str());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_array() -> Result<()> {
        let ctx = Context::default();
        let val = Value::array(ctx.inner());
        assert!(val.is_ok());
        assert!(val.unwrap().is_array());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_object() -> Result<()> {
        let ctx = Context::default();
        let val = Value::object(ctx.inner());
        assert!(val.is_ok());
        assert!(val.unwrap().is_object());
        Ok(())
    }

    #[test]
    fn test_allows_representing_a_value_as_f64() -> Result<()> {
        let ctx = Context::default();
        let val = Value::from_f64(ctx.inner(), f64::MIN)?.as_f64()?;
        assert_eq!(val, f64::MIN);
        Ok(())
    }

    #[test]
    fn test_value_as_str() {
        let s = "hello";
        let ctx = Context::default();
        let val = Value::from_str(ctx.inner(), s).unwrap();
        assert_eq!(val.as_str().unwrap(), s);
    }

    #[test]
    fn test_value_as_str_middle_nul_terminator() {
        let s = "hello\0world!";
        let ctx = Context::default();
        let val = Value::from_str(ctx.inner(), s).unwrap();
        assert_eq!(val.as_str().unwrap(), s);
    }

    #[test]
    fn test_exception() {
        let ctx = Context::default();
        let error = ctx
            .eval_global("main", "should_throw")
            .unwrap_err()
            .to_string();
        let expected_error =
            "Uncaught ReferenceError: \'should_throw\' is not defined\n    at <eval> (main)\n";
        assert_eq!(expected_error, error.as_str());
    }

    #[test]
    fn test_exception_with_stack() {
        let ctx = Context::default();
        let script = r#"
            function foo() { return bar(); }
            function bar() { return foobar(); }
            function foobar() {
                for (var i = 0; i < 100; i++) {
                    if (i > 25) {
                        throw new Error("boom");
                    }
                }
            }
            foo();
        "#;
        let expected_error = r#"Uncaught Error: boom
    at foobar (main:7)
    at bar (main)
    at foo (main)
    at <eval> (main:11)
"#;
        let error = ctx.eval_global("main", script).unwrap_err().to_string();
        assert_eq!(expected_error, error.as_str());
    }

    #[test]
    fn test_syntax_error() {
        let ctx = Context::default();
        let error = ctx
            .eval_global("main", "func boom() {}")
            .unwrap_err()
            .to_string();
        let expected_error = "Uncaught SyntaxError: expecting \';\'\n    at main:1\n";
        assert_eq!(expected_error, error.as_str());
    }
}
