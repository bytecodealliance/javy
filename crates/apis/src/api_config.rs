/// A configuration for APIs added in this crate.
///
/// Example usage:
/// ```
/// # use javy_apis::APIConfig;
/// let api_config = APIConfig::default();
/// ```
#[derive(Debug, Default)]
pub struct APIConfig {
    #[cfg(feature = "console")]
    pub(crate) console: crate::console::ConsoleConfig,
}
