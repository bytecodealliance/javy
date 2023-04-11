use std::{fmt, convert::TryInto, collections::HashMap};

use anyhow::Result;

use crate::js_binding::value::JSValueRef;
use super::{js_value::JSValue, convert::from_qjs_value};

#[derive(Copy, Clone)]
pub struct CallbackArg {
    inner: JSValueRef
}

impl CallbackArg {
    pub fn new(inner: JSValueRef) -> Self {
        Self { inner }
    }

    fn to_js_value(&self) -> Result<JSValue> {
        from_qjs_value(&self.inner.get_context_ref(), &self.inner)
    }
}

impl fmt::Display for CallbackArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_js_value().unwrap())
    }
}

macro_rules! try_into_impl {
    ($($t:ty),+ $(,)?) => {
        $(impl TryInto<$t> for &CallbackArg {
            type Error = anyhow::Error;

            fn try_into(self) -> Result<$t> {
                self.to_js_value()?.try_into()
            }
        }

        impl TryInto<$t> for CallbackArg {
            type Error = anyhow::Error;

            fn try_into(self) -> Result<$t> {
                self.to_js_value()?.try_into()
            }
        })+
    };
}

try_into_impl!(
    bool,
    i32,
    usize,
    f64,
    String,
    Vec<JSValue>,
    Vec<u8>,
    HashMap<String, JSValue>,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js_binding::context::JSContextRef;

    #[test]
    fn test_callback_arg() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42").unwrap();
        let callback_arg = &CallbackArg { inner: val };
        let arg: i32 = callback_arg.try_into()?;
        assert_eq!(arg, 42);
        Ok(())
    }
}