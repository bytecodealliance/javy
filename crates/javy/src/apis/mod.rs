//! A collection of APIs for Javy.
//!
//! APIs are enabled through the the [`Config`](crate::Config).
//!
//! Example usage:
//! ```rust
//!
//! use javy::{Config, Runtime, from_js_error};
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     let mut config = Config::default();
//!     config.text_encoding(true);
//!     let runtime = Runtime::new(config)?;
//!     let context = runtime.context();
//!     context.with(|cx| {
//!         cx.eval_with_options::<(), _>(
//!             r#"
//!                 console.log(new TextEncoder().encode(""))
//!             "#,
//!             Default::default()
//!         )
//!         .map_err(|e| from_js_error(cx.clone(), e))
//!     })?;
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
//! Disabled by default.
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
//! Disabled by default. Only available when targeting WASI preview 1 since it
//! will always error when targeting later WASI previews.
//!
//! ###  `JSON`
//! Provides an efficient implementation of JSON functions based on [`simd-json`](https://crates.io/crates/simd-json/0.13.10)
//! and [`serde_json`](https://crates.io/crates/serde_json)
//!
//! Disabled by default.
pub(crate) mod console;
#[cfg(feature = "json")]
pub(crate) mod json;
pub(crate) mod random;
#[cfg(all(target_family = "wasm", target_os = "wasi", target_env = "p1"))]
pub(crate) mod stream_io;
pub(crate) mod text_encoding;
