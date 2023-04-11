use anyhow::Result;

use crate::js_binding::value::JSValueRef;
use super::{js_value::JSValue, convert::from_qjs_value};

pub struct CallbackArg {
    inner: JSValueRef
}

impl CallbackArg {
    pub fn new(inner: JSValueRef) -> Self {
        Self { inner }
    }

    pub fn value(&self) -> Result<JSValue> {
        from_qjs_value(&self.inner.get_context_ref(), &self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js_binding::context::JSContextRef;

    #[test]
    fn test_callback_arg() {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42").unwrap();
        let arg = CallbackArg { inner: val };
        assert_eq!(arg.value().unwrap(), JSValue::Int(42));
    }
}