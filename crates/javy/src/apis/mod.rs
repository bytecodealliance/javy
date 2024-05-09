//! A collection of APIs for Javy.
//!
//! APIs are enabled through the the [`Config`](crate::Config) and are defined
//! in term of the [`Intrinsic`](rquickjs::context::Intrinsic) provided by
//! rquickjs.
//!
//! Example usage:
//! ```rust
//!
//! use javy::{Runtime, from_js_error};
//! use javy_apis::RuntimeExt;
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     let mut config = Config::default();
//!     config.text_decoding(true);
//!     let runtime = Runtime::new(config);
//!     let context = runtime.context();
//!     context.with(|cx| {
//!         cx.eval_with_options(Default::default(), r#
//!           "console.log(new TextEncdoder().decode(""))
//!         "#)
//!         .map_err(|e| to_js_error(cx.clone(), e))?
//!     });
//!     Ok(())
//! }
//!
//! ```
//!
//! ## Features
//!
//! ### `console`
//!
//! Adds an implementation of the `console.log` and `console.error`.
//!
//! ### `TextEncoding`
//!
//! Provides partial implementations of `TextEncoder` and `TextDecoder`.
//! Disables by default.
//!
//! ### `Random`
//!
//! Overrides the implementation of `Math.random` to one that seeds
//! the RNG on first call to `Math.random`. This is helpful to enable when using
//! using a tool like Wizer to snapshot a [`Runtime`] so that the output of
//! `Math.random` relies on the WASI context used at runtime and not the WASI
//! context used when snapshotting.
//!
//! ### `StreamIO`
//!
//! Provides an implementation of `Javy.IO.readSync` and `Javy.IO.writeSync`.
//! Disabled by default.

pub(crate) mod console;
pub(crate) mod random;
pub(crate) mod stream_io;
pub(crate) mod text_encoding;

pub(crate) use console::*;
pub(crate) use random::*;
pub(crate) use stream_io::*;
pub(crate) use text_encoding::*;
