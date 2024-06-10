use anyhow::Result;
use javy::{Config, Runtime};

pub(crate) fn new_runtime() -> Result<Runtime> {
    let mut config = Config::default();
    let config = config
        .text_encoding(true)
        .redirect_stdout_to_stderr(true)
        .javy_stream_io(true)
        .override_json_parse_and_stringify(true)
        .javy_json(true);

    Runtime::new(std::mem::take(config))
}
