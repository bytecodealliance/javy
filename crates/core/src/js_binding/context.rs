#![allow(dead_code)]
use super::value::Value;
use anyhow::Result;
use quickjs_sys::{
    JSContext, JSRuntime, JSValue, JS_Call, JS_Eval, JS_GetGlobalObject, JS_NewContext,
    JS_NewRuntime, JS_EVAL_TYPE_GLOBAL,
};
use std::ffi::CString;

#[derive(Debug)]
pub struct Context {
    runtime: *mut JSRuntime,
    inner: *mut JSContext,
}

impl Default for Context {
    fn default() -> Self {
        let runtime = unsafe { JS_NewRuntime() };
        if runtime.is_null() {
            panic!("Couldn't create JavaScript runtime");
        }

        let inner = unsafe { JS_NewContext(runtime) };
        if inner.is_null() {
            panic!("Couldn't create JavaScript context");
        }

        Self { runtime, inner }
    }
}

impl Context {
    pub fn eval_global(&self, name: &str, contents: &str) -> Result<Value> {
        let input = CString::new(contents)?;
        let script_name = CString::new(name)?;
        let len = contents.len() - 1;
        let raw = unsafe {
            JS_Eval(
                self.inner,
                input.as_ptr(),
                len as _,
                script_name.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        Value::new(self.inner, raw)
    }

    pub fn inner(&self) -> *mut JSContext {
        self.inner
    }

    pub fn global_object(&self) -> Result<Value> {
        let raw = unsafe { JS_GetGlobalObject(self.inner) };
        Value::new(self.inner, raw)
    }

    pub(crate) fn call(&self, fun: &Value, receiver: &Value, args: &[Value]) -> Result<Value> {
        let inner_args: Vec<JSValue> = args.iter().map(|v| v.inner()).collect();
        let return_val = unsafe {
            JS_Call(
                self.inner,
                fun.inner(),
                receiver.inner(),
                args.len() as i32,
                inner_args.as_slice().as_ptr() as *mut JSValue,
            )
        };

        Value::new(self.inner, return_val)
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use anyhow::Result;
    const SCRIPT_NAME: &str = "context.js";

    #[test]
    fn test_new_returns_a_context() -> Result<()> {
        let _ = Context::default();
        Ok(())
    }

    #[test]
    fn test_context_evalutes_code_globally() -> Result<()> {
        let ctx = Context::default();
        let contents = "var a = 1;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_reports_invalid_code() -> Result<()> {
        let ctx = Context::default();
        let contents = "a + 1 * z;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_err());
        Ok(())
    }

    #[test]
    fn test_context_allows_access_to_global_object() -> Result<()> {
        let ctx = Context::default();
        let val = ctx.global_object();
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_allows_calling_a_function() -> Result<()> {
        let ctx = Context::default();
        let contents = "globalThis.foo = function() { return 1; }";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let fun = global.get_property("foo")?;
        let result = ctx.call(&fun, &global, &[]);
        assert!(result.is_ok());
        Ok(())
    }
}
