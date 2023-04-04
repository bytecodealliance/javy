use std::io;

use anyhow::Result;
use quickjs_wasm_rs::{Context, Value};

mod globals;

#[derive(Debug)]
pub struct Runtime {
    context: Context,
}

impl Runtime {
    pub fn default() -> Result<Self> {
        let context= Context::default();

        #[cfg(feature = "console")]
        globals::console::add(&context)?;

        Ok(Self {
           context
        })
    }

    pub fn eval(&self, name: &str, contents: &str) -> Result<Value> {
        self.context.eval_global(name, contents)
    }

    pub fn set_global(&self, name: &str, value: Value) -> Result<()> {
        self.context.global_object()?.set_property(name, value)
    }
    
    pub fn compile_module(&self, name: &str, contents: &str) -> Result<Vec<u8>> {
        self.context.compile_module(name, contents)
    }

    pub fn run_bytecode(&self, bytecode: &[u8]) -> Result<Value> {
        self.context.eval_binary(bytecode)
    }
}