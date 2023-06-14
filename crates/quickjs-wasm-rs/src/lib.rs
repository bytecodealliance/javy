//! High-level bindings and serializers for a Wasm build of QuickJS.

//! ## Bindings

//! `JSContextRef` corresponds to a QuickJS `JSContext` and `JSValueRef` corresponds to a QuickJS `JSValue`.

//! ```
//! use quickjs_wasm_rs::JSContextRef;

//! let mut context = JSContextRef::default();
//! ```

//! will create a new context.
//!
//! ## Callbacks
//! To create a callback to be used in JavaScript, use `wrap_callback`:
//! ```
//! use quickjs_wasm_rs::js_value::JSValue;
//! let context = JSContextRef::default();
//! let callback = context.wrap_callback(|_ctx, _this, args| {
//!    let s = args[0].to_string();
//!    println!("{}", s);
//!    Ok(JSValue::Undefined)
//! })?;
//! let global = context.global_object()?;
//! global.set_property("print", callback)?;
//! ```
//!
//! ### Converting to and from Rust types
//! When working with callbacks, it is often useful to convert to Rust types.
//!
//! `_this` and `args` in the callback function are of type `JSValueRef` which can be converted into `JSValue`.
//! `JSValue` supports `try_into` to convert to Rust types.
//!
//! Rust types can then be converted back to `JSValue` using `try_into`.
//! ```
//! use quickjs_wasm_rs::js_value::JSValue;
//!
//! ctx.wrap_callback(|_ctx, this, args| {
//!     let this: std::collections::HashMap<String, JSValue> = this.try_into()?;
//!     let first_arg: Vec<JSValue> = args[0].try_into()?;
//!     let ret_val = 0;
//!     Ok(ret_val.try_into()?)
//! })?;
//! ```

//! ## Serializers

//! This crate provides optional transcoding features for converting between
//! serialization formats and `JSValueRef`:
//! - `messagepack` provides `quickjs_wasm_rs::messagepack` for msgpack, using `rmp_serde`.
//! - `json` provides `quickjs_wasm_rs::json` for JSON, using `serde_json`.

//! msgpack example:

//! ```rust
//! use quickjs_wasm_rs::{messagepack, JSContextRef, JSValueRef};

//! let context = JSContextRef::default();
//! let input_bytes: &[u8] = ...;
//! let input_value = messagepack::transcode_input(&context, input_bytes).unwrap();
//! let output_value: JSValueRef = ...;
//! let output = messagepack::transcode_output(output_value).unwrap();
//! ```
mod js_binding;
mod js_value;
mod serialize;

pub use crate::js_binding::context::JSContextRef;
pub use crate::js_binding::error::JSError;
pub use crate::js_binding::exception::Exception;
pub use crate::js_binding::value::JSValueRef;
pub use crate::js_value::qjs_convert::*;
pub use crate::js_value::JSValue;
pub use crate::serialize::de::Deserializer;
pub use crate::serialize::ser::Serializer;
