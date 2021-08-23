use super::value::Value;
use anyhow::{anyhow, Result};
use quickjs_sys::{
    JSContext, JSRuntime, JS_Eval, JS_NewContext, JS_NewRuntime, JS_EVAL_TYPE_GLOBAL,
    JS_GetGlobalObject, JS_Call, JSValue
};
use std::ffi::CString;

#[derive(Debug)]
pub(crate) struct Context {
    runtime: *mut JSRuntime,
    inner: *mut JSContext,
}

impl Context {
    pub(crate) fn new() -> Result<Self> {
        let runtime = unsafe { JS_NewRuntime() };
        if runtime.is_null() {
            return Err(anyhow!("Couldn't create JavaScript runtime"));
        }

        let inner = unsafe { JS_NewContext(runtime) };
        if inner.is_null() {
            return Err(anyhow!("Couldn't create JavaScript context"));
        }

        Ok(Self { runtime, inner })
    }

    pub(crate) fn inner(&self) -> *mut JSContext {
        self.inner
    }

    pub(crate) fn eval_global(&self, name: &str, contents: &str) -> Result<Value> {
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

        Value::new(self, raw)
    }

    pub(crate) fn global_object(&self) -> Result<Value> {
        let raw = unsafe { JS_GetGlobalObject(self.inner) };
        Value::new(self, raw)
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

        Value::new(self, return_val)
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use anyhow::Result as R;
    const SCRIPT_NAME: &str = "context.js";

    #[test]
    fn test_new_returns_a_context() -> R<()> {
        let ctx = Context::new();
        assert!(ctx.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_evalutes_code_globally() -> R<()> {
        let ctx = Context::new()?;
        let contents = "var a = 1;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_reports_invalid_code() -> R<()> {
        let ctx = Context::new()?;
        let contents = "a + 1 * z;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_err());
        Ok(())
    }

    #[test]
    fn test_context_allows_access_to_global_object() -> R<()> {
        let ctx = Context::new()?;
        let val = ctx.global_object();
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_allows_calling_a_function() -> R<()> {
        let ctx = Context::new()?;
        let contents = "globalThis.foo = function() { return 1; }";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let fun = global.get_property("foo")?;
        let result = ctx.call(&fun, &global, &[]);
        assert!(result.is_ok());
        Ok(())
    }
}
