use std::io::{self, Read};

use javy_plugin_api::javy::Runtime;
use javy_plugin_api::Config;

use crate::exports::bytecodealliance::javy_default_plugin::invokable;
use crate::exports::bytecodealliance::javy_plugin::javy_plugin_exports;
use crate::shared_config::SharedConfig;

mod shared_config;

wit_bindgen::generate!({ world: "javy-default-plugin", generate_all });

struct Component;

impl Guest for Component {
    fn config_schema() -> Vec<u8> {
        shared_config::config_schema()
    }
}

impl invokable::Guest for Component {
    fn invoke(bytecode: Vec<u8>, function: Option<String>) -> () {
        javy_plugin_api::invoke(&bytecode, function.as_deref())
    }
}

impl javy_plugin_exports::Guest for Component {
    fn compile_src(src: Vec<u8>) -> Vec<u8> {
        javy_plugin_api::compile_src(&src)
    }

    fn initialize_runtime() -> () {
        javy_plugin_api::reinitialize_runtime(config, modify_runtime).unwrap();
    }
}

export!(Component);

fn config() -> Config {
    // Read shared config JSON in from stdin.
    // Using stdin instead of an environment variable because the value set for
    // an environment variable will persist as the value set for that environment
    // variable in subsequent invocations so a different value can't be used to
    // initialize a runtime with a different configuration.
    let mut config = Config::default();
    config
        .text_encoding(true)
        .javy_stream_io(true)
        .simd_json_builtins(true);

    let mut config_bytes = vec![];
    let shared_config = match io::stdin().read_to_end(&mut config_bytes) {
        Ok(0) => None,
        Ok(_) => Some(SharedConfig::parse_from_json(&config_bytes).unwrap()),
        Err(e) => panic!("Error reading from stdin: {e}"),
    };
    if let Some(shared_config) = shared_config {
        shared_config.apply_to_config(&mut config);
    }
    config
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime
}
