//! Configurable JavaScript runtime for WebAssembly
//!
//! Example usage:
//! ```
//! # use javy::{Config, Runtime};
//! let runtime = Runtime::new(Config::default()).unwrap();
//! runtime.context().eval_global("test.js", "console.log('hello world!');").unwrap();
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
