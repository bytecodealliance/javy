//! Configurable JavaScript runtime for WebAssembly.
//!
//! Example usage:
//! ```
//! # use anyhow::anyhow;
//! # use javy::{quickjs::JSValue, Runtime};
//! let runtime = Runtime::default();
//! let context = runtime.context();
//! context
//!     .global_object()
//!     .unwrap()
//!     .set_property(
//!         "print",
//!         context
//!             .wrap_callback(move |_ctx, _this, args| {
//!                 let str = args
//!                     .first()
//!                     .ok_or(anyhow!("Need to pass an argument"))?
//!                     .to_string();
//!                 println!("{str}");
//!                 Ok(JSValue::Undefined)
//!             })
//!             .unwrap(),
//!     )
//!     .unwrap();
//! context.eval_global("hello.js", "print('hello!');").unwrap();
//! ```
//!
//! ## Core concepts
//! * [`Runtime`] - The entrypoint for using the JavaScript runtime. Use a
//!   [`Config`] to configure behavior.
//!
//! ## Features
//! * `json` - functions for converting between [`quickjs::JSValueRef`] and JSON
//!   byte slices
//! * `messagepack` - functions for converting between [`quickjs::JSValueRef`]
//!   and MessagePack byte slices

pub use config::Config;
pub use quickjs_wasm_rs as quickjs;
pub use runtime::Runtime;

pub mod alloc;
mod config;
mod runtime;

#[cfg(feature = "messagepack")]
pub mod messagepack;

#[cfg(feature = "json")]
pub mod json;
