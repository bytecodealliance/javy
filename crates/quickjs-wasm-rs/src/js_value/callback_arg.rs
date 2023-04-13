use std::{collections::HashMap, convert::TryInto, fmt};

use anyhow::Result;

use super::{convert::from_qjs_value, js_value::JSValue};
use crate::js_binding::value::JSValueRef;

#[derive(Copy, Clone)]
pub struct CallbackArg<'a> {
    inner: JSValueRef<'a>,
}

impl<'a> CallbackArg<'a> {
    pub fn new(inner: JSValueRef<'a>) -> CallbackArg<'a> {
        Self { inner }
    }

    pub unsafe fn inner_value(&self) -> JSValueRef {
        self.inner
    }

    fn to_js_value(&self) -> Result<JSValue> {
        from_qjs_value(self.inner.context, &self.inner)
    }
}

impl fmt::Display for CallbackArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_js_value().unwrap())
    }
}

macro_rules! try_into_impl {
    ($($t:ty),+ $(,)?) => {
        $(impl TryInto<$t> for &CallbackArg<'_> {
            type Error = anyhow::Error;

            fn try_into(self) -> Result<$t> {
                self.to_js_value()?.try_into()
            }
        }

        impl TryInto<$t> for CallbackArg<'_> {
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
