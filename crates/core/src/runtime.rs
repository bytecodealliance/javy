use anyhow::Result;
use javy::{Config, Runtime};
use javy_config::Config as SharedConfig;

pub(crate) fn new(shared_config: SharedConfig) -> Result<Runtime> {
    let mut config = Config::default();
    let config = config
        .text_encoding(shared_config.contains(SharedConfig::TEXT_ENCODING))
        .redirect_stdout_to_stderr(shared_config.contains(SharedConfig::REDIRECT_STDOUT_TO_STDERR))
        .javy_stream_io(shared_config.contains(SharedConfig::JAVY_STREAM_IO))
        .override_json_parse_and_stringify(
            shared_config.contains(SharedConfig::OVERRIDE_JSON_PARSE_AND_STRINGIFY),
        )
        .javy_json(shared_config.contains(SharedConfig::JAVY_JSON));

    Runtime::new(std::mem::take(config))
}
