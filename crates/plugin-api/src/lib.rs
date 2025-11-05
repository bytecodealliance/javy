//! A crate for creating Javy plugins
//!
//! Example usage for creating WASI preview2 plugins:
//! ```ignore
//! use javy_plugin_api::{
//!     javy::{quickjs::prelude::Func, Runtime},
//!     javy_plugin, Config,
//! };
//!
//! wit_bindgen::generate!({ world: "my-javy-plugin-v1", generate_all });
//!
//! fn config() -> Config {
//!     let mut config = Config::default();
//!     config
//!         .text_encoding(true)
//!         .javy_stream_io(true);
//!     config
//! }
//!
//! fn modify_runtime(runtime: Runtime) -> Runtime {
//!     runtime.context().with(|ctx| {
//!         ctx.globals().set("plugin", true).unwrap();
//!     });
//!     runtime
//! }
//!
//! struct Component;
//!
//! // Dynamically linked modules will use `my_javy_plugin_v1` as the import
//! // namespace.
//! javy_plugin!("my-javy-plugin-v1", Component, config, modify_runtime);
//!
//! export!(Component);
//! ```
//!
//! //! Example for creating WASI preview 1 plugins:
//! ```ignore
//! use javy_plugin_api::{
//!     import_namespace,
//!     javy::{quickjs::prelude::Func, Runtime},
//!     Config,
//! };
//!
//! import_namespace!("test-plugin-wasip1");
//!
//! #[link(wasm_import_module = "some_host")]
//! extern "C" {
//!     fn imported_function();
//! }
//!
//! fn config() -> Config {
//!     Config::default()
//! }
//!
//! fn modify_runtime(runtime: Runtime) -> Runtime {
//!     runtime.context().with(|ctx| {
//!         ctx.globals().set("plugin", true).unwrap();
//!         ctx.globals()
//!             .set(
//!                 "func",
//!                 Func::from(|| {
//!                     unsafe { crate::imported_function() };
//!                 }),
//!             )
//!             .unwrap();
//!     });
//!     runtime
//! }
//!
//! #[export_name = "initialize-runtime"]
//! fn initialize_runtime() {
//!     javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap()
//! }
//! ```
//!
//! The crate will automatically add exports for a number of Wasm functions in
//! your crate that Javy needs to work.
//!
//! # Core concepts
//! * [`javy`] - a re-export of the [`javy`] crate.
//! * [`javy_plugin`] - Used for WASI preview 2 plugins and not WASI preview 1
//!   plugins. Takes a namespace to use for the module name for imports, a
//!   struct to add function exports to, a config method, and a method for
//!   updating the Javy runtime.
//! * [`import_namespace`] - Used for WASI preview 1 plugins. Takes a namespace
//!   to use for the module name for imports.
//! * [`Config`] - to add behavior to the created [`javy::Runtime`].
//!
//! # Features
//! * `json` - enables the `json` feature in the `javy` crate.
//! * `messagepack` - enables the `messagepack` feature in the `javy` crate.

// Allow these in this file because we only run this program single threaded
// and we can safely reason about the accesses to the Javy Runtime. We also
// don't want to introduce overhead from taking unnecessary mutex locks.
#![allow(static_mut_refs)]
use anyhow::{anyhow, bail, Result};
pub use config::Config;
use javy::quickjs::{self, Ctx, Error as JSError, Function, Module, Value};
use javy::{from_js_error, Runtime};
use std::cell::OnceCell;
use std::str;

pub use javy;

mod config;
mod javy_plugin;
mod namespace;
#[cfg(all(target_family = "wasm", target_os = "wasi", target_env = "p1"))]
mod wasi_p1;

const FUNCTION_MODULE_NAME: &str = "function.mjs";

thread_local! {
    static COMPILE_SRC_RET_AREA: OnceCell<[u32; 2]> = const { OnceCell::new() }
}

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();
static mut EVENT_LOOP_ENABLED: bool = false;

static EVENT_LOOP_ERR: &str = r#"
                Pending jobs in the event queue.
                Scheduling events is not supported when the 
                event-loop runtime config is not enabled.
            "#;

/// Initializes the Javy runtime.
pub fn initialize_runtime<F, G>(config: F, modify_runtime: G) -> Result<()>
where
    F: FnOnce() -> Config,
    G: FnOnce(Runtime) -> Runtime,
{
    let config = config();
    let runtime = Runtime::new(config.runtime_config)?;
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
/// Returns result with the success value being a vector of the bytecode and
/// failure being the error message.
///
/// # Arguments
///
/// * `config` - A function that returns a config for Javy
/// * `modify_runtime` - A function that returns a Javy runtime
/// * `js_src` - A slice of bytes representing the JS source code
pub fn compile_src(js_src: &[u8]) -> Result<Vec<u8>> {
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
    runtime.compile_to_bytecode(FUNCTION_MODULE_NAME, &String::from_utf8_lossy(js_src))
}

/// Evaluates QuickJS bytecode and optionally invokes exported JS function with
/// name.
///
/// # Arguments
///
/// * `bytecode` - The QuickJS bytecode
/// * `fn_name` - The JS function name
pub fn invoke(bytecode: &[u8], fn_name: Option<&str>) -> Result<()> {
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
