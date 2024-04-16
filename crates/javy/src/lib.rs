//! Configurable JavaScript runtime for WebAssembly.
//!
//! Example usage:
//! ```
//! use anyhow::anyhow;
//! use javy::{Runtime, from_js_error};
//! let runtime = Runtime::default();
//! let context = runtime.context();
//!
//! context.with(|cx| {
//!     let globals = this.globals();
//!     globals.set(
//!         "print_hello",
//!         Function::new(
//!             this.clone(),
//!             MutFn::new(move |_, _| {
//!                 println!("Hello, world!");
//!             }),
//!         )?,
//!     )?;
//! });
//!
//! context.with(|cx| {
//!     cx.eval_with_options(Default::default(), "print_hello();")
//!         .map_err(|e| from_js_error(cx.clone(), e))
//!         .map(|_| ())
//! });
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
use std::str;

pub mod alloc;
mod config;
mod runtime;
mod serde;

use anyhow::{anyhow, Error, Result};
use rquickjs::{
    prelude::Rest, qjs, Ctx, Error as JSError, Exception, String as JSString, Type, Value,
};
use std::fmt::Write;

#[cfg(feature = "messagepack")]
pub mod messagepack;

#[cfg(feature = "json")]
pub mod json;

/// Print the given JS value.
///
/// The implementation matches the default JavaScript display format for each value.
pub fn print(val: &Value, sink: &mut String) -> Result<()> {
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
        x => unimplemented!("{x}"),
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

/// Alias for `Args::hold(cx, args).release()`
#[macro_export]
macro_rules! hold_and_release {
    ($cx:expr, $args:expr) => {
        Args::hold($cx, $args).release()
    };
}

/// Alias for [Args::hold]
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

/// Converts the JavaScript value to a string, replacing any invalid UTF-8 sequences with the
/// Unicode replacement character (U+FFFD).
// TODO: Upstream this?
pub fn to_string_lossy<'js>(cx: &Ctx<'js>, string: &JSString<'js>, error: JSError) -> String {
    let mut len: qjs::size_t = 0;
    let ptr = unsafe { qjs::JS_ToCStringLen2(cx.as_raw().as_ptr(), &mut len, string.as_raw(), 0) };
    let buffer = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    // The error here *must* be a Utf8 error; the `JSString::to_string()` may
    // return `JSError::Unknown`, but at that point, something else has gone
    // wrong too.

    let mut utf8_error = match error {
        JSError::Utf8(e) => e,
        _ => unreachable!("expected Utf8 error"),
    };
    let mut res = String::new();
    let mut buffer = buffer;
    loop {
        let (valid, after_valid) = buffer.split_at(utf8_error.valid_up_to());
        res.push_str(unsafe { str::from_utf8_unchecked(valid) });
        res.push(char::REPLACEMENT_CHARACTER);

        // see https://simonsapin.github.io/wtf-8/#surrogate-byte-sequence
        let lone_surrogate = matches!(after_valid, [0xED, 0xA0..=0xBF, 0x80..=0xBF, ..]);

        // https://simonsapin.github.io/wtf-8/#converting-wtf-8-utf-8 states that each
        // 3-byte lone surrogate sequence should be replaced by 1 UTF-8 replacement
        // char. Rust's `Utf8Error` reports each byte in the lone surrogate byte
        // sequence as a separate error with an `error_len` of 1. Since we insert a
        // replacement char for each error, this results in 3 replacement chars being
        // inserted. So we use an `error_len` of 3 instead of 1 to treat the entire
        // 3-byte sequence as 1 error instead of as 3 errors and only emit 1
        // replacement char.
        let error_len = if lone_surrogate {
            3
        } else {
            utf8_error
                .error_len()
                .expect("Error length should always be available on underlying buffer")
        };

        buffer = &after_valid[error_len..];
        match str::from_utf8(buffer) {
            Ok(valid) => {
                res.push_str(valid);
                break;
            }
            Err(e) => utf8_error = e,
        }
    }
    res
}
