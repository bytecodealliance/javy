use crate::{Config, Runtime};
use anyhow::Result;
use javy_config::Config as SharedConfig;
use std::{slice, str};

const FUNCTION_MODULE_NAME: &str = "function.mjs";

static mut COMPILE_SRC_RET_AREA: [u32; 2] = [0; 2];

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn initialize_runtime<F>(config: Option<Config>, extend_runtime_fn: F)
where
    F: FnOnce(Runtime) -> Runtime,
{
    let js_runtime_config = config.unwrap_or_else(|| user_config().unwrap_or_default());
    let mut runtime = Runtime::new(js_runtime_config.into()).unwrap();
    runtime = extend_runtime_fn(runtime);
    unsafe {
        RUNTIME.take();
        RUNTIME
            .set(runtime)
            // `set` requires `T` to implement `Debug` but quickjs::{Runtime,
            // Context} don't.
            .map_err(|_| anyhow!("Could not pre-initialize javy::Runtime"))
            .unwrap();
    }
}

pub fn user_config() -> Option<Config> {
    env::var("JS_RUNTIME_CONFIG")
        .map(|js_runtime_config_str| {
            SharedConfig::from_bits(
                js_runtime_config_str
                    .parse()
                    .expect("JS_RUNTIME_CONFIG should be a u32"),
            )
            .expect("JS_RUNTIME_CONFIG should contain only valid flags")
        })
        .ok()
        .map(|c| c.into())
}

/// Compiles JS source code to QuickJS bytecode.
///
/// Returns a pointer to a buffer containing a 32-bit pointer to the bytecode byte array and the
/// u32 length of the bytecode byte array.
///
/// # Arguments
///
/// * `js_src_ptr` - A pointer to the start of a byte array containing UTF-8 JS source code
/// * `js_src_len` - The length of the byte array containing JS source code
///
/// # Safety
///
/// * `js_src_ptr` must reference a valid array of unsigned bytes of `js_src_len` length
#[export_name = "compile_src"]
pub unsafe extern "C" fn compile_src(js_src_ptr: *const u8, js_src_len: usize) -> *const u32 {
    let runtime = RUNTIME.take().unwrap();
    let js_src = str::from_utf8(slice::from_raw_parts(js_src_ptr, js_src_len)).unwrap();

    let bytecode = runtime
        .compile_to_bytecode(FUNCTION_MODULE_NAME, js_src)
        .unwrap();

    // We need the bytecode buffer to live longer than this function so it can be read from memory
    let len = bytecode.len();
    let bytecode_ptr = Box::leak(bytecode.into_boxed_slice()).as_ptr();
    COMPILE_SRC_RET_AREA[0] = bytecode_ptr as u32;
    COMPILE_SRC_RET_AREA[1] = len.try_into().unwrap();
    COMPILE_SRC_RET_AREA.as_ptr()
}

/// Evaluates QuickJS bytecode and invokes the exported JS function name.
///
/// # Safety
///
/// * `bytecode_ptr` must reference a valid array of bytes of `bytecode_len`
///   length.
/// * `fn_name_ptr` must reference a UTF-8 string with `fn_name_len` byte
///   length.
#[export_name = "invoke"]
pub unsafe extern "C" fn invoke(
    bytecode_ptr: *const u8,
    bytecode_len: usize,
    fn_name_ptr: *const u8,
    fn_name_len: usize,
) {
    let runtime = RUNTIME.get().unwrap();
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    run_bytecode(runtime, bytecode);
    if !fn_name_ptr.is_null() && fn_name_len != 0 {
        let fn_name = str::from_utf8_unchecked(slice::from_raw_parts(fn_name_ptr, fn_name_len));
        invoke_function(runtime, FUNCTION_MODULE_NAME, fn_name);
    }
}

use std::{cell::OnceCell, env, process};

use crate::{
    from_js_error,
    quickjs::{context::EvalOptions, Ctx, Error as JSError, Module, Value},
    to_js_error,
};
use anyhow::{anyhow, bail, Error};

static EVENT_LOOP_ERR: &str = r#"
                Pending jobs in the event queue.
                Scheduling events is not supported when the 
                experimental_event_loop cargo feature is disabled.
            "#;

/// Evaluate the given bytecode.
///
/// Evaluating also prepares (or "instantiates") the state of the JavaScript
/// engine given all the information encoded in the bytecode.
fn run_bytecode(runtime: &Runtime, bytecode: &[u8]) {
    runtime
        .context()
        .with(|this| {
            let module = unsafe { Module::load(this.clone(), bytecode)? };
            let (_, promise) = module.eval()?;

            handle_maybe_promise(this.clone(), promise.into())
        })
        .map_err(|e| runtime.context().with(|cx| from_js_error(cx.clone(), e)))
        .and_then(|_: ()| ensure_pending_jobs(runtime))
        .unwrap_or_else(handle_error)
}

/// Entry point to invoke an exported JavaScript function.
///
/// This function will evaluate a JavaScript snippet that imports and invokes
/// the target function from a previously evaluated module. It's the caller's
/// reponsibility to ensure that the module containing the target function has
/// been previously evaluated.
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
            opts.strict = false;
            opts.global = false;
            let value = this.eval_with_options::<Value<'_>, _>(js, opts)?;

            handle_maybe_promise(this.clone(), value)
        })
        .map_err(|e| runtime.context().with(|cx| from_js_error(cx.clone(), e)))
        .and_then(|_: ()| ensure_pending_jobs(runtime))
        .unwrap_or_else(handle_error)
}

/// Handles the promise returned by evaluating the JS bytecode.
fn handle_maybe_promise(this: Ctx, value: Value) -> crate::quickjs::Result<()> {
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
