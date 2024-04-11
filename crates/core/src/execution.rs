use std::process;

use anyhow::{bail, Error, Result};
use javy::{
    from_js_error,
    quickjs::{context::EvalOptions, Module, Value},
    Runtime,
};

/// Evaluate the given bytecode.
///
/// Evaluating also prepares (or "instantiates") the state of the JavaScript
/// engine given all the information encoded in the bytecode.
// TODO: Rename this?
pub fn run_bytecode(runtime: &Runtime, bytecode: &[u8]) {
    runtime
        .context()
        .with(|this| Module::instantiate_read_object(this, bytecode).and_then(|_| Ok(())))
        .map_err(|e| runtime.context().with(|cx| from_js_error(cx.clone(), e)))
        // Prefer calling `process_event_loop` *outside* of the `with` callback,
        // to avoid errors regarding multiple mutable borrows.
        .and_then(|_| process_event_loop(runtime))
        .unwrap_or_else(handle_error)
}

/// Entry point to invoke an exported JavaScript function.
///
/// This function will evaluate a JavaScript snippet that imports and invokes
/// the target function from a previously evaluated module. It's the caller's
/// reponsibility to ensure that the module containing the target function has
/// been previously evaluated.
// TODO:
// * Consider renaming this to emit_invoke_function.
// * There's probably(?) a better way of doing this through rquickjs' module
// loaders and resolvers, in such a way that we can avoid having to eval.
pub fn invoke_function(runtime: &Runtime, fn_module: &str, fn_name: &str) {
    let js = if fn_name == "default" {
        format!("import {{ default as defaultFn }} from '{fn_module}'; defaultFn();")
    } else {
        format!("import {{ {fn_name} }} from '{fn_module}'; {fn_name}();")
    };

    runtime
        .context()
        .with(|this| {
            let mut opts = EvalOptions::default();
            // TODO: config?
            opts.strict = false;
            // We're assuming imports and exports, therefore we want to
            // evaluate as a module.
            opts.global = false;
            this.eval_with_options::<Value<'_>, _>(js.into_bytes(), opts)
                .map_err(|e| from_js_error(this.clone(), e))
                .and_then(|_| Ok(()))
        })
        .and_then(|_: ()| process_event_loop(&runtime))
        .unwrap_or_else(handle_error)
}

fn process_event_loop(rt: &Runtime) -> Result<()> {
    // TODO:
    // I'm not sure if having a cargo feature gives us a lot here, given that
    // we're not preventing the usage of the event loop at the engine level; it
    // might be worth revisiting this to make it a config option instead and
    // including / excluding the right engine intrinsics via rquickjs
    if cfg!(feature = "experimental_event_loop") {
        rt.resolve_pending_jobs()?
    } else if rt.has_pending_jobs() {
        bail!(
            r#"
                Pending jobs in the event queue.
                Scheduling events is not supported when the 
                experimental_event_loop cargo feature is disabled.
            "#
        );
    }

    Ok(())
}

fn handle_error(e: Error) {
    eprintln!("{e}");
    process::abort();
}
