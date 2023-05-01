use super::{context::JSContextRef, value::JSValueRef};
use anyhow::{anyhow, Result};
use quickjs_wasm_sys::{JS_GetException, JS_IsError};
use std::fmt;

/// `Exception` represents a JavaScript exception that occurs within the QuickJS context.
///
/// This struct provides a convenient way to capture and handle JavaScript exceptions that
/// may be thrown during the execution of JavaScript code. It includes the error message and
/// an optional stack trace to help with debugging and error reporting.
///
/// # Example
///
/// ```
/// // Assuming you have a `JSContextRef` where an exception has been thrown.
/// let exception = Exception::new(context)?;
/// let err = Err(exception.into_error());
/// ```
#[derive(Debug)]
pub struct Exception {
    msg: String,
    stack: Option<String>,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        if let Some(stack) = &self.stack {
            write!(f, "\n{stack}")?;
        }
        Ok(())
    }
}

impl Exception {
    pub(super) fn new(context: &JSContextRef) -> Result<Self> {
        let exception_value = unsafe { JS_GetException(context.inner) };
        Self::from(JSValueRef::new_unchecked(context, exception_value))
    }

    pub fn from(exception_obj: JSValueRef) -> Result<Self> {
        let msg = exception_obj.as_str().map(ToString::to_string)?;
        let mut stack = None;

        let is_error = unsafe { JS_IsError(exception_obj.context.inner, exception_obj.value) } != 0;
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
