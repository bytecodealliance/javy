use super::context::Context;
use anyhow::{anyhow, Result};
use quickjs_sys::{JSValue, JS_TAG_EXCEPTION, JS_GetPropertyStr};
use std::ffi::CString;

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

    pub(crate) fn inner(&self) -> JSValue {
        self.value
    }

    pub(crate) fn get_property(&self, key: &str) -> Result<Value> {
        let cstring_key = CString::new(key)?;
        let raw = unsafe {
            JS_GetPropertyStr(self.context.inner(), self.value, cstring_key.as_ptr())
        };

        Value::new(self.context, raw)
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
    use super::Context;
    use anyhow::Result as R;
    const SCRIPT_NAME: &str = "value.js";

    #[test]
    fn test_value_objects_allow_retrieving_a_property() -> R<()> {
        let ctx = Context::new()?;
        let contents = "globalThis.bar = 1;";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let prop = global.get_property("bar");
        assert!(prop.is_ok());
        Ok(())
    }
}
