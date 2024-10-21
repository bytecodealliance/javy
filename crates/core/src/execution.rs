use std::process;

use anyhow::{anyhow, bail, Error, Result};
use javy::{
    from_js_error,
    quickjs::{Ctx, Error as JSError, Function, Module, Value},
    to_js_error, Runtime,
};

static EVENT_LOOP_ERR: &str = r#"
                Pending jobs in the event queue.
                Scheduling events is not supported when the 
                experimental_event_loop cargo feature is disabled.
            "#;

/// Evaluate the given bytecode.
///
/// Evaluating also prepares (or "instantiates") the state of the JavaScript
/// engine given all the information encoded in the bytecode.
pub fn run_bytecode(runtime: &Runtime, bytecode: &[u8], fn_name: Option<&str>) {
    runtime
        .context()
        .with(|this| {
            let module = unsafe { Module::load(this.clone(), bytecode)? };
            let (module, promise) = module.eval()?;

            handle_maybe_promise(this.clone(), promise.into())?;

            if let Some(fn_name) = fn_name {
                let fun: Function = module.get(fn_name)?;
                // Exported functions are guaranteed not to have arguments so
                // we can safely pass an empty tuple for arguments.
                let value = fun.call(())?;
                handle_maybe_promise(this.clone(), value)?
            }
            Ok(())
        })
        .map_err(|e| runtime.context().with(|cx| from_js_error(cx.clone(), e)))
        .and_then(|_: ()| ensure_pending_jobs(runtime))
        .unwrap_or_else(handle_error)
}

/// Handles the promise returned by evaluating the JS bytecode.
fn handle_maybe_promise(this: Ctx, value: Value) -> javy::quickjs::Result<()> {
    match value.as_promise() {
        Some(promise) => {
            if cfg!(feature = "experimental_event_loop") {
                // If the experimental event loop is enabled, trigger it.
                let resolved = promise.finish::<Value>();
                // `Promise::finish` returns Err(Wouldblock) when the all
                // pending jobs have been handled.
                if let Err(JSError::WouldBlock) = resolved {
                    Ok(())
                } else {
                    resolved.map(|_| ())
                }
            } else {
                // Else we simply expect the promise to resolve immediately.
                match promise.result() {
                    None => Err(to_js_error(this, anyhow!(EVENT_LOOP_ERR))),
                    Some(r) => r,
                }
            }
        }
        None => Ok(()),
    }
}

fn ensure_pending_jobs(rt: &Runtime) -> Result<()> {
    if cfg!(feature = "experimental_event_loop") {
        rt.resolve_pending_jobs()
    } else if rt.has_pending_jobs() {
        bail!(EVENT_LOOP_ERR);
    } else {
        Ok(())
    }
}

fn handle_error(e: Error) {
    eprintln!("{e}");
    process::abort();
}
