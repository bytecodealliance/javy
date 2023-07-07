use std::process;

use anyhow::{bail, Error, Result};
use javy::{quickjs::JSContextRef, Runtime};

pub fn run_bytecode(runtime: &Runtime, bytecode: &[u8]) {
    let context = runtime.context();
    context
        .eval_binary(bytecode)
        .and_then(|_| process_event_loop(context))
        .unwrap_or_else(handle_error);
}

pub fn invoke_function(runtime: &Runtime, fn_module: &str, fn_name: &str) {
    let context = runtime.context();
    let js = if fn_name == "default" {
        format!("import {{ default as defaultFn }} from '{fn_module}'; defaultFn();")
    } else {
        format!("import {{ {fn_name} }} from '{fn_module}'; {fn_name}();")
    };
    context
        .eval_module("runtime.mjs", &js)
        .and_then(|_| process_event_loop(context))
        .unwrap_or_else(handle_error);
}

fn process_event_loop(context: &JSContextRef) -> Result<()> {
    if cfg!(feature = "experimental_event_loop") {
        context.execute_pending()?;
    } else if context.is_pending() {
        bail!("Adding tasks to the event queue is not supported");
    }
    Ok(())
}

fn handle_error(e: Error) {
    eprintln!("Error while running JS: {e}");
    process::abort();
}
