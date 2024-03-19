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
//!
//! If you want to customize the runtime or the APIs, you can use the
//! [`Runtime::new_with_apis`] method instead to provide a [`javy::Config`]
//! for the underlying [`Runtime`] or an [`APIConfig`] for the APIs.
//!
//! ## Features
//! * `console` - Registers an implementation of the `console` API.
//! * `text_encoding` - Registers implementations of `TextEncoder` and `TextDecoder`.
//! * `random` - Overrides the implementation of `Math.random` to one that
//!   seeds the RNG on first call to `Math.random`. This is helpful to enable
//!   when using Wizer to snapshot a [`javy::Runtime`] so that the output of
//!   `Math.random` relies on the WASI context used at runtime and not the
//!   WASI context used when Wizening. Enabling this feature will increase the
//!   size of the Wasm module that includes the Javy Runtime and will
//!   introduce an additional hostcall invocation when `Math.random` is
//!   invoked for the first time.
//! * `stream_io` - Registers implementations of `Javy.IO.readSync` and `Javy.IO.writeSync`.

use anyhow::Result;
use javy::Runtime;

use javy::quickjs::{Type, Value};
use std::fmt::Write;

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

/// Print the given JS value.
///
/// The implementation matches the default JavaScript display format for each value.
pub(crate) fn print<'js>(val: &Value<'js>, sink: &mut String) -> Result<()> {
    match val.type_of() {
        Type::Undefined => write!(sink, "undefined").map_err(Into::into),
        Type::Null => write!(sink, "null").map_err(Into::into),
        Type::Bool => {
            let b = val.as_bool().unwrap();
            write!(sink, "{}", b).map_err(Into::into)
        }
        Type::Int => {
            let i = val.as_int().unwrap();
            write!(sink, "{}", i).map_err(Into::into)
        }
        Type::Float => {
            let f = val.as_float().unwrap();
            write!(sink, "{}", f).map_err(Into::into)
        }
        Type::String => {
            let s = val.as_string().unwrap();
            write!(sink, "{}", s.to_string()?).map_err(Into::into)
        }
        Type::Array => {
            let inner = val.as_array().unwrap();
            for e in inner.iter() {
                print(&e?, sink)?
            }
            Ok(())
        }
        Type::Object => write!(sink, "[object Object]").map_err(Into::into),
        // TODO: Implement the rest.
        _ => unimplemented!(),
    }
}
