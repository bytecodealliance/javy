//! Configurable JavaScript runtime for WebAssembly.
//!
//! Example usage:
//! ```
//! # use javy::{Config, Runtime};
//! let runtime = Runtime::default();
//! let context = runtime.context();
//! context.global_object().unwrap().set_property(
//!     "print",
//!     context.wrap_callback(move |ctx, _this, args| {
//!         let str = args.first().unwrap().to_string();
//!         println!("{str}");
//!         Ok(javy::quickjs::from_qjs_value(&ctx.undefined_value().unwrap()).unwrap())
//!     }).unwrap(),
//! ).unwrap();
//! context.eval_global("hello.js", "print('hello!');").unwrap();
//! ```
//!
//! ## Core concepts
//! * [`Runtime`] - The entrypoint for using the JavaScript runtime. Use a
//!   [`Config`] to configure behavior.

pub use config::Config;
pub use quickjs_wasm_rs as quickjs;
pub use runtime::Runtime;

mod config;
mod runtime;
