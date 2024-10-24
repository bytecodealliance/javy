use anyhow::Result;
use javy::{Runtime, SharedConfig};

pub(crate) fn new(shared_config: SharedConfig) -> Result<Runtime> {
    // let mut config = Config::default();
    // let config = config
    //     .text_encoding(shared_config.contains(SharedConfig::TEXT_ENCODING))
    //     .redirect_stdout_to_stderr(shared_config.contains(SharedConfig::REDIRECT_STDOUT_TO_STDERR))
    //     .javy_stream_io(shared_config.contains(SharedConfig::JAVY_STREAM_IO))
    //     .simd_json_builtins(shared_config.contains(SharedConfig::SIMD_JSON_BUILTINS))
    //     .javy_json(shared_config.contains(SharedConfig::JAVY_JSON));

    Runtime::new(shared_config.into())
}
