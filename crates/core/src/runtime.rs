use anyhow::Result;
use javy::{Config, Runtime};
use javy_config::Config as SharedConfig;

pub(crate) fn new(shared_config: SharedConfig) -> Result<Runtime> {
    let mut config = Config::default();
    let config = config
        .text_encoding(shared_config.contains(SharedConfig::TEXT_ENCODING))
        .redirect_stdout_to_stderr(shared_config.contains(SharedConfig::REDIRECT_STDOUT_TO_STDERR))
        .javy_stream_io(shared_config.contains(SharedConfig::JAVY_STREAM_IO))
        // Due to an issue with our custom serializer and property accesses
        // we're disabling this temporarily. It will be enabled once we have a
        // fix forward.
        .override_json_parse_and_stringify(true)
        .javy_json(true);

    Runtime::new(std::mem::take(config))
}
