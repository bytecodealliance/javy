use std::io::{self, Read};
use std::process;

use javy_plugin_api::javy::Runtime;
use javy_plugin_api::{import_namespace, Config};

use crate::shared_config::SharedConfig;

mod shared_config;

wit_bindgen::generate!({ world: "javy-default-plugin", generate_all });

import_namespace!("javy-default-plugin-v1");

struct Component;

impl Guest for Component {
    fn config_schema() -> Vec<u8> {
        shared_config::config_schema()
    }

    fn invoke(bytecode: Vec<u8>, function: Option<String>) {
        javy_plugin_api::invoke(&bytecode, function.as_deref()).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::abort();
        })
    }

    fn compile_src(src: Vec<u8>) -> Result<Vec<u8>, String> {
        javy_plugin_api::compile_src(&src).map_err(|e| e.to_string())
    }

    fn initialize_runtime() {
        javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap()
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
