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

#![deny(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls
)]

use anyhow::Result;
use javy::Runtime;

pub use api_config::APIConfig;
pub use runtime_ext::RuntimeExt;

mod api_config;
mod runtime_ext;

pub(crate) trait JSApiSet {
    fn register(&self, runtime: &Runtime, config: APIConfig) -> Result<()>;
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
pub fn add_to_runtime(_runtime: &Runtime, _config: APIConfig) -> Result<()> {
    Ok(())
}
