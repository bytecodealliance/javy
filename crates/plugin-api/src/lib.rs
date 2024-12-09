//! A crate for creating Javy plugins
//!
//! Example usage:
//! ```rust
//! use javy_plugin_api::import_namespace;
//! use javy_plugin_api::Config;
//!
//! // Dynamically linked modules will use `my_javy_plugin_v1` as the import
//! // namespace.
//! import_namespace!("my_javy_plugin_v1");
//!
//! #[export_name = "initialize_runtime"]
//! pub extern "C" fn initialize_runtime() {
//!    let mut config = Config::default();
//!    config
//!        .text_encoding(true)
//!        .javy_stream_io(true);
//!
//!    javy_plugin_api::initialize_runtime(config, |runtime| runtime).unwrap();
//! }
//! ```
//!
//! The crate will automatically add exports for a number of Wasm functions in
//! your crate that Javy needs to work.
//!
//! # Core concepts
//! * [`javy`] - a re-export of the [`javy`] crate.
//! * [`import_namespace`] - required to provide an import namespace when the
//!   plugin is used to generate dynamically linked modules.
//! * [`initialize_runtime`] - used to configure the QuickJS runtime with a
//!   [`Config`] to add behavior to the created [`javy::Runtime`].
//!
//! # Features
//! * `json` - enables the `json` feature in the `javy` crate.

// Allow these in this file because we only run this program single threaded
// and we can safely reason about the accesses to the Javy Runtime. We also
// don't want to introduce overhead from taking unnecessary mutex locks.
#![allow(static_mut_refs)]
use anyhow::{anyhow, bail, Error, Result};
pub use config::Config;
use javy::quickjs::{self, Ctx, Error as JSError, Function, Module, Value};
use javy::{from_js_error, Runtime};
use std::cell::OnceCell;
use std::{process, slice, str};

pub use javy;

mod config;
mod namespace;

const FUNCTION_MODULE_NAME: &str = "function.mjs";

static mut COMPILE_SRC_RET_AREA: [u32; 2] = [0; 2];

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();
static mut EVENT_LOOP_ENABLED: bool = false;

static EVENT_LOOP_ERR: &str = r#"
                Pending jobs in the event queue.
                Scheduling events is not supported when the 
                event-loop runtime config is not enabled.
            "#;

/// Initializes the Javy runtime.
pub fn initialize_runtime<F>(config: Config, modify_runtime: F) -> Result<()>
where
    F: FnOnce(Runtime) -> Runtime,
{
    let runtime = Runtime::new(config.runtime_config).unwrap();
    let runtime = modify_runtime(runtime);
    unsafe {
        RUNTIME.take(); // Allow re-initializing.
        RUNTIME
            .set(runtime)
            // `unwrap` requires error `T` to implement `Debug` but `set`
            // returns the `javy::Runtime` on error and `javy::Runtime` does not
            // implement `Debug`.
            .map_err(|_| anyhow!("Could not pre-initialize javy::Runtime"))
            .unwrap();
        EVENT_LOOP_ENABLED = config.event_loop;
    };
    Ok(())
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
    // Use initialized runtime when compiling because certain runtime
    // configurations can cause different bytecode to be emitted.
    //
    // For example, given the following JS:
    // ```
    // function foo() {
    //   "use math"
    //   1234 % 32
    // }
    // ```
    //
    // Setting `config.bignum_extension` to `true` will produce different
    // bytecode than if it were set to `false`.
    let runtime = unsafe { RUNTIME.get().unwrap() };
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

/// Evaluates QuickJS bytecode and optionally invokes exported JS function with
/// name.
///
/// # Safety
///
/// * `bytecode_ptr` must reference a valid array of bytes of `bytecode_len`
///   length.
/// * If `fn_name_ptr` is not 0, it must reference a UTF-8 string with
///   `fn_name_len` byte length.
#[export_name = "invoke"]
pub unsafe extern "C" fn invoke(
    bytecode_ptr: *const u8,
    bytecode_len: usize,
    fn_name_ptr: *const u8,
    fn_name_len: usize,
) {
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    let fn_name = if !fn_name_ptr.is_null() && fn_name_len != 0 {
        Some(str::from_utf8_unchecked(slice::from_raw_parts(
            fn_name_ptr,
            fn_name_len,
        )))
    } else {
        None
    };
    run_bytecode(bytecode, fn_name);
}

/// Evaluate the given bytecode.
///
/// Deprecated for use outside of this crate.
///
/// Evaluating also prepares (or "instantiates") the state of the JavaScript
/// engine given all the information encoded in the bytecode.
pub fn run_bytecode(bytecode: &[u8], fn_name: Option<&str>) {
    let runtime = unsafe { RUNTIME.get() }.unwrap();
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
fn handle_maybe_promise(this: Ctx, value: Value) -> quickjs::Result<()> {
    match value.as_promise() {
        Some(promise) => {
            if unsafe { EVENT_LOOP_ENABLED } {
                // If the event loop is enabled, trigger it.
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
                    None => Err(javy::to_js_error(this, anyhow!(EVENT_LOOP_ERR))),
                    Some(r) => r,
                }
            }
        }
        None => Ok(()),
    }
}

fn ensure_pending_jobs(rt: &Runtime) -> Result<()> {
    if unsafe { EVENT_LOOP_ENABLED } {
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
