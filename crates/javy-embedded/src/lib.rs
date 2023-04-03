use std::io;

use anyhow::Result;
use quickjs_wasm_rs::{Context, Value};

mod globals;

pub struct Runtime {
    ctx: Context,
}

impl Runtime {
    pub fn default() -> Result<Self> {
        let ctx= Context::default();
        globals::inject_javy_globals(&ctx,io::stdout(), io::stderr())?;
        Ok(Self {
           ctx
        })
    }

    pub fn eval(&self, name: &str, contents: &str) -> Result<Value> {
        self.ctx.eval_global(name, contents)
    }
}