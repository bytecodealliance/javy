use bitflags::bitflags;

use crate::Config;

#[export_name = "config_simd_json_builtins"]
pub static mut SIMD_JSON_BUILTINS: i32 = -1;

#[export_name = "config_javy_json"]
pub static mut JAVY_JSON: i32 = -1;

#[export_name = "config_javy_stream_io"]
pub static mut JAVY_STREAM_IO: i32 = -1;

#[export_name = "config_redirect_stdout_to_stderr"]
pub static mut REDIRECT_STDOUT_TO_STDERR: i32 = -1;

#[export_name = "config_text_encoding"]
pub static mut TEXT_ENCODING: i32 = -1;

bitflags! {
    #[derive(Eq, PartialEq, Debug)]
    pub struct SharedConfig: u32 {
        const SIMD_JSON_BUILTINS = 1;
        const JAVY_JSON = 1 << 1;
        const JAVY_STREAM_IO = 1 << 2;
        const REDIRECT_STDOUT_TO_STDERR = 1 << 3;
        const TEXT_ENCODING = 1 << 4;
    }
}

impl Default for SharedConfig {
    fn default() -> Self {
        let mut config = SharedConfig::empty();
        config.set(SharedConfig::SIMD_JSON_BUILTINS, true);
        config.set(SharedConfig::JAVY_JSON, true);
        config.set(SharedConfig::JAVY_STREAM_IO, true);
        config.set(SharedConfig::REDIRECT_STDOUT_TO_STDERR, true);
        config.set(SharedConfig::TEXT_ENCODING, true);
        config
    }
}

impl From<SharedConfig> for Config {
    fn from(value: SharedConfig) -> Self {
        let mut config = Self::default();
        #[cfg(feature = "json")]
        config.simd_json_builtins(value.contains(SharedConfig::SIMD_JSON_BUILTINS));
        #[cfg(feature = "json")]
        config.javy_json(value.contains(SharedConfig::JAVY_JSON));
        config.javy_stream_io(value.contains(SharedConfig::JAVY_STREAM_IO));
        config.redirect_stdout_to_stderr(value.contains(SharedConfig::REDIRECT_STDOUT_TO_STDERR));
        config.text_encoding(value.contains(SharedConfig::TEXT_ENCODING));
        config
    }
}

#[export_name = "generate_config"]
pub unsafe extern "C" fn generate_config() -> u32 {
    let mut config = SharedConfig::default();

    set_property(
        &mut config,
        SIMD_JSON_BUILTINS,
        SharedConfig::SIMD_JSON_BUILTINS,
        "SIMD_JSON_BUILTINS",
    );
    set_property(&mut config, JAVY_JSON, SharedConfig::JAVY_JSON, "JAVY_JSON");
    set_property(
        &mut config,
        JAVY_STREAM_IO,
        SharedConfig::JAVY_STREAM_IO,
        "JAVY_STREAM_IO",
    );
    set_property(
        &mut config,
        REDIRECT_STDOUT_TO_STDERR,
        SharedConfig::REDIRECT_STDOUT_TO_STDERR,
        "REDIRECT_STDOUT_TO_STDERR",
    );
    set_property(
        &mut config,
        TEXT_ENCODING,
        SharedConfig::TEXT_ENCODING,
        "TEXT_ENCODING",
    );

    config.bits()
}

fn set_property(config: &mut SharedConfig, global: i32, prop: SharedConfig, name: &str) {
    if global == 0 {
        config.set(prop, false);
    } else if global == 1 {
        config.set(prop, true);
    } else if global != -1 {
        panic!("Invalid {name} value");
    }
}
