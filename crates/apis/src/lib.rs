//! JS APIs for Javy.
//!
//! This crate provides JS APIs you can add to Javy.
//!
//! Example usage:
//! ```
//! # use anyhow::{anyhow, Error, Result};
//! use javy::{quickjs::JSValue, Runtime};
//! use javy_apis::RuntimeExt;
//!
//! let runtime = Runtime::new_with_defaults()?;
//! let context = runtime.context();
//! context.global_object()?.set_property(
//!    "print",
//!    context.wrap_callback(move |_ctx, _this, args| {
//!        let str = args
//!            .first()
//!            .ok_or(anyhow!("Need to pass an argument"))?
//!            .to_string();
//!        println!("{str}");
//!        Ok(JSValue::Undefined)
//!    })?,
//! )?;
//! context.eval_global("hello.js", "print('hello!');")?;
//! # Ok::<(), Error>(())
//! ```

use anyhow::Result;
use javy::Runtime;

pub use api_config::APIConfig;
#[cfg(feature = "console")]
pub use console::LogStream;
pub use runtime_ext::RuntimeExt;

mod api_config;
#[cfg(feature = "console")]
mod console;
mod runtime_ext;
#[cfg(feature = "stream_io")]
mod stream_io;

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
    #[cfg(feature = "stream_io")]
    stream_io::StreamIO::new().register(runtime, &config)?;
    Ok(())
}
