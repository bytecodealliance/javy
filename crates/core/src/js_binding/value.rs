use super::context::Context;
use anyhow::{anyhow, Result};
use quickjs_sys::{JSValue, JS_TAG_EXCEPTION};

#[derive(Debug)]
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
}

fn get_tag(v: JSValue) -> i32 {
    (v >> 32) as i32
}

fn is_exception(t: i32) -> bool {
    matches!(t, JS_TAG_EXCEPTION)
}
