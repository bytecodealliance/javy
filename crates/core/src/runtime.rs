use anyhow::Result;
use javy::{Config, Runtime};
use javy_apis::{APIConfig, LogStream, RuntimeExt};

pub(crate) fn new_runtime() -> Result<Runtime> {
    let mut api_config = APIConfig::default();
    api_config.log_stream(LogStream::StdErr);
    Runtime::new_with_apis(Config::default(), api_config)
}
