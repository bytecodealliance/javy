use anyhow::Result;
use javy::{Config, Runtime};
use javy_apis::{APIConfig, LogStream};

pub(crate) fn runtime() -> Result<Runtime> {
    let runtime = Runtime::new(&Config::default())?;

    let mut api_config = APIConfig::default();
    api_config.log_stream(LogStream::StdErr);
    javy_apis::add_to_runtime(&runtime, &api_config)?; // uses `wasmtime_wasi::sync::add_to_linker` approach

    Ok(runtime)
}
