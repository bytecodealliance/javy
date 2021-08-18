use super::value::Value;
use anyhow::{anyhow, Result};
use quickjs_sys::{
    JSContext, JSRuntime, JS_Eval, JS_NewContext, JS_NewRuntime, JS_EVAL_TYPE_GLOBAL,
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

    pub(crate) fn eval_global(&self, name: &str, contents: &[u8]) -> Result<Value> {
        let input = c_string_from_bytes(contents)?;
        let len = contents.len() - 1;
        let raw = unsafe {
            JS_Eval(
                self.inner,
                input.as_ptr(),
                len as _,
                // not sure why this is ok in the previous
                // impl but here I need to cast to *const i8
                name.as_ptr() as *const i8,
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        Value::new(self, raw)
    }
}

fn c_string_from_bytes(bytes: impl Into<Vec<u8>>) -> Result<CString> {
    let cstring = CString::new(bytes)?;
    Ok(cstring)
}

#[cfg(test)]
mod tests {
    use super::Context;
    use anyhow::Result as R;

    #[test]
    fn test_new_returns_a_context() -> R<()> {
        let ctx = Context::new();
        assert!(ctx.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_evalutes_code_globally() -> R<()> {
        let ctx = Context::new()?;
        let name = "script.js";
        let contents = "var a = 1;";
        let val = ctx.eval_global(&name, contents.as_bytes());
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_reports_invalid_code() -> R<()> {
        let ctx = Context::new()?;
        let name = "script.js";
        let contents = "a + 1 * z;";
        let val = ctx.eval_global(&name, contents.as_bytes());
        assert!(val.is_err());
        Ok(())
    }
}
