use anyhow::Result;
use javy::{Config, Runtime};

pub(crate) fn new_runtime() -> Result<Runtime> {
    let mut config = Config::default();
    let config = config
        .text_encoding(true)
        .redirect_stdout_to_stderr(true)
        .javy_stream_io(true);

    Runtime::new(std::mem::take(config))
}
