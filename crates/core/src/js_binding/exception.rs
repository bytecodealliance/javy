use super::value::Value;
use anyhow::{anyhow, Result};
use quickjs_wasm_sys::{JSContext, JS_GetException, JS_IsError};
use std::fmt;

#[derive(Debug)]
pub struct Exception {
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

impl Exception {
    pub(super) fn new(context: *mut JSContext) -> Result<Self> {
        let exception_value = unsafe { JS_GetException(context) };
        let exception_obj = Value::new_unchecked(context, exception_value);

        let msg = exception_obj.as_str().map(ToString::to_string)?;
        let mut stack = None;

        let is_error = unsafe { JS_IsError(context, exception_value) } != 0;
        if is_error {
            let stack_value = exception_obj.get_property("stack")?;
            if !stack_value.is_undefined() {
                stack.replace(stack_value.as_str().map(ToString::to_string)?);
            }
        }

        Ok(Exception { msg, stack })
    }

    pub fn into_error(self) -> anyhow::Error {
        anyhow!("Uncaught {}", self)
    }
}
