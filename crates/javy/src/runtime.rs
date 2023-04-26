use crate::quickjs::JSContextRef;
use anyhow::Result;

use crate::Config;

/// A JavaScript Runtime.
///
/// Provides a [`Self::context()`] method for working with the underlying [`JSContextRef`].
///
/// ## Examples
///
/// ```
/// # use javy::{Config, Runtime};
/// let runtime = Runtime::new(Config::default()).unwrap();
/// runtime.context().eval_global("test.js", "console.log('hello world!');").unwrap();
/// ```
#[derive(Debug)]
pub struct Runtime {
    context: JSContextRef,
}

impl Runtime {
    /// Creates a new [`Runtime`].
    pub fn new(_config: Config) -> Result<Self> {
        let context = JSContextRef::default();
        Ok(Self { context })
    }

    /// A reference to a [`JSContextRef`].
    pub fn context(&self) -> &JSContextRef {
        &self.context
    }
}

impl Default for Runtime {
    /// Returns a [`Runtime`] with a default configuration. Panics if there's
    /// an error.
    fn default() -> Self {
        Self::new(Config::default()).unwrap()
    }
}
