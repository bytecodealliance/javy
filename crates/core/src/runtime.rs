use anyhow::Result;
use javy::{Config, LogStream, Runtime};

pub(crate) fn runtime() -> Result<Runtime> {
    let mut config = Config::default();
    config.log_stream(LogStream::StdErr);
    Ok(Runtime::new(&config)?)
}
