use anyhow::Result;
use javy::{Config, Runtime};

use crate::APIConfig;

/// A extension trait for [`Runtime`] that creates a [`Runtime`] with APIs
/// provided in this crate.
///
/// ## Example
/// ```
/// # use anyhow::Error;
/// use javy::Runtime;
/// use javy_apis::RuntimeExt;
/// let runtime = Runtime::new_with_defaults()?;
/// # Ok::<(), Error>(())
/// ```
pub trait RuntimeExt {
    /// Creates a [`Runtime`] configured by the provided [`Config`] with JS
    /// APIs added configured according to the [`APIConfig`].
    fn new_with_apis(config: Config, api_config: APIConfig) -> Result<Runtime>;
    /// Creates a [`Runtime`] with JS APIs added with a default configuration.
    fn new_with_defaults() -> Result<Runtime>;
}

impl RuntimeExt for Runtime {
    fn new_with_apis(config: Config, api_config: APIConfig) -> Result<Runtime> {
        let runtime = Runtime::new(config)?;
        crate::add_to_runtime(&runtime, api_config)?;
        Ok(runtime)
    }

    fn new_with_defaults() -> Result<Runtime> {
        Self::new_with_apis(Config::default(), APIConfig::default())
    }
}
