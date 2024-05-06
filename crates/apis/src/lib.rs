//! A collection of APIs for Javy.
//!
//! APIs are enabled through cargo features.
//!
//! Example usage:
//! ```rust
//!
//! //With the `console` feature enabled.
//! use javy::{Runtime, from_js_error};
//! use javy_apis::RuntimeExt;
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     let runtime = Runtime::new_with_defaults()?;
//!     let context = runtime.context();
//!     context.with(|cx| {
//!         cx.eval_with_options(Default::default(), "console.log('hello!');")
//!             .map_err(|e| to_js_error(cx.clone(), e))?
//!     });
//!     Ok(())
//! }
//!
//! ```
//!
//! If you want to customize the runtime or the APIs, you can use the
//! [`Runtime::new_with_apis`] method instead to provide a [`javy::Config`]
//! for the underlying [`Runtime`] or an [`APIConfig`] for the APIs.
//!
//! ## Features
//! * `console`:  Adds an implementation of the `console.log` and `console.error`,
//! enabling the configuration of the standard streams.
//! * `text_encoding`:  Registers implementations of `TextEncoder` and `TextDecoder`.
//! * `random`: Overrides the implementation of `Math.random` to one that seeds
//! the RNG on first call to `Math.random`. This is helpful to enable when using
//! using a tool like Wizer to snapshot a [`Runtime`] so that the output of
//! `Math.random` relies on the WASI context used at runtime and not the WASI
//! context used when Wizening. Enabling this feature will increase the size of
//! the Wasm module that includes the Javy Runtime and will introduce an
//! additional hostcall invocation when `Math.random` is invoked for the first
//! time.
//! * `stream_io`: Adds an implementation of `Javy.IO.readSync` and `Javy.IO.writeSync`.

use anyhow::Result;
use javy::Runtime;

pub use api_config::APIConfig;
#[cfg(feature = "console")]
pub use console::LogStream;
pub use runtime_ext::RuntimeExt;

mod api_config;
#[cfg(feature = "console")]
mod console;
#[cfg(feature = "random")]
mod random;
mod runtime_ext;
#[cfg(feature = "stream_io")]
mod stream_io;
#[cfg(feature = "text_encoding")]
mod text_encoding;

pub(crate) trait JSApiSet {
    fn register(&self, runtime: &Runtime, config: &APIConfig) -> Result<()>;
}

/// Adds enabled JS APIs to the provided [`Runtime`].
///
/// ## Example
/// ```
/// # use anyhow::Error;
/// # use javy::Runtime;
/// # use javy_apis::APIConfig;
/// let runtime = Runtime::default();
/// javy_apis::add_to_runtime(&runtime, APIConfig::default())?;
/// # Ok::<(), Error>(())
/// ```
pub fn add_to_runtime(runtime: &Runtime, config: APIConfig) -> Result<()> {
    #[cfg(feature = "console")]
    console::Console::new().register(runtime, &config)?;
    #[cfg(feature = "random")]
    random::Random.register(runtime, &config)?;
    #[cfg(feature = "stream_io")]
    stream_io::StreamIO.register(runtime, &config)?;
    #[cfg(feature = "text_encoding")]
    text_encoding::TextEncoding.register(runtime, &config)?;
    Ok(())
}
