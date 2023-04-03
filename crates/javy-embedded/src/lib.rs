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
        globals::inject_javy_globals(&context,io::stdout(), io::stderr())?;
        Ok(Self {
           context
        })
    }
    
    pub fn no_globals() -> Self {
        let context= Context::default();
        Self {
           context
        }
    }

    pub fn eval(&self, name: &str, contents: &str) -> Result<Value> {
        self.context.eval_global(name, contents)
    }

    pub fn compile_module(&self, name: &str, contents: &str) -> Result<Vec<u8>> {
        self.context.compile_module(name, contents)
    }

    pub fn run_bytecode(&self, bytecode: &[u8]) -> Result<Value> {
        self.context.eval_binary(bytecode)
    }
}