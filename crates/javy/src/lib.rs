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
//! * `export_alloc_fns` - exports [`alloc::canonical_abi_realloc`] and
//!   [`alloc::canonical_abi_free`] from generated WebAssembly for allocating
//!   and freeing memory
//! * `json` - functions for converting between [`quickjs::JSValueRef`] and JSON
//!   byte slices
//! * `messagepack` - functions for converting between [`quickjs::JSValueRef`]
//!   and MessagePack byte slices

pub use config::Config;
pub use rquickjs as quickjs;
pub use runtime::Runtime;

pub mod alloc;
mod config;
mod runtime;

use anyhow::{anyhow, Error, Result};
use rquickjs::{prelude::Rest, Ctx, Error as JSError, Exception, Type, Value};
use std::fmt::Write;

#[cfg(feature = "messagepack")]
pub mod messagepack;

#[cfg(feature = "json")]
pub mod json;

/// Print the given JS value.
///
/// The implementation matches the default JavaScript display format for each value.
pub fn print<'js>(val: &Value<'js>, sink: &mut String) -> Result<()> {
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

/// A struct to hold the current [Ctx] and [Value]s passed as arguments to Rust
/// functions.
/// A struct here is used to explicitly tie these values with a particular
/// lifetime.
//
// See: https://github.com/rust-lang/rfcs/pull/3216
pub struct Args<'js>(Ctx<'js>, Rest<Value<'js>>);

impl<'js> Args<'js> {
    /// Tie the [Ctx] and [Rest<Value>].
    pub fn hold(cx: Ctx<'js>, args: Rest<Value<'js>>) -> Self {
        Self(cx, args)
    }

    /// Get the [Ctx] and [Rest<Value>].
    pub fn release(self) -> (Ctx<'js>, Rest<Value<'js>>) {
        (self.0, self.1)
    }
}

/// Alias for `Args::hold(cx, args).release()
#[macro_export]
macro_rules! hold_and_release {
    ($cx:expr, $args:expr) => {
        Args::hold($cx, $args).release()
    };
}

/// Alias for `Args::hold(cx, args)
#[macro_export]
macro_rules! hold {
    ($cx:expr, $args:expr) => {
        Args::hold($cx, $args)
    };
}

/// Handles a JavaScript error or exception and converts to [anyhow::Error].
pub fn from_js_error(ctx: Ctx<'_>, e: JSError) -> Error {
    if e.is_exception() {
        let exception = ctx.catch().into_exception().unwrap();
        anyhow!("{exception}")
    } else {
        Into::into(e)
    }
}

/// Converts an [anyhow::Error]  to a [JSError].
///
/// If the error is an [anyhow::Error] this function will construct and throw
/// a JS [Exception] in order to construct the [JSError].
pub fn to_js_error(cx: Ctx, e: Error) -> JSError {
    match e.downcast::<JSError>() {
        Ok(e) => e,
        Err(e) => cx.throw(Value::from_exception(
            Exception::from_message(cx.clone(), &e.to_string())
                .expect("creating an exception to succeed"),
        )),
    }
}
