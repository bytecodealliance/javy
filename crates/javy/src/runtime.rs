use anyhow::Result;
use quickjs_wasm_rs::JSContextRef;

use crate::Config;

#[derive(Debug)]
pub struct Runtime {
    context: JSContextRef,
}

impl Runtime {
    pub fn new(#[allow(unused_variables)] config: &Config) -> Result<Self> {
        let context = JSContextRef::default();
        Ok(Self { context })
    }

    pub fn context(&self) -> &JSContextRef {
        &self.context
    }
}
