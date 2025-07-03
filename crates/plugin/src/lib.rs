use javy_plugin_api::javy::Runtime;
use javy_plugin_api::{import_namespace, Config};

use crate::bindings::exports::bytecodealliance::javy_plugin::javy_plugin_exports::Guest;

mod bindings;
mod shared_config;

import_namespace!("javy_quickjs_provider_v3");

struct Component;

impl Guest for Component {
    fn compile_src(src: Vec<u8>) -> Vec<u8> {
        javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap();
        javy_plugin_api::compile_src(&src)
    }

    fn initialize_runtime() -> () {
        javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap();
    }

    fn invoke(bytecode: Vec<u8>, function: Option<String>) -> () {
        javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap();
        javy_plugin_api::invoke(&bytecode, function.as_deref())
    }

    fn config_schema() -> Vec<u8> {
        shared_config::config_schema()
    }
}

bindings::export!(Component with_types_in bindings);

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

    // let mut config_bytes = vec![];
    // let shared_config = match io::stdin().read_to_end(&mut config_bytes) {
    //     Ok(0) => None,
    //     Ok(_) => Some(SharedConfig::parse_from_json(&config_bytes).unwrap()),
    //     Err(e) => panic!("Error reading from stdin: {e}"),
    // };
    // if let Some(shared_config) = shared_config {
    //     shared_config.apply_to_config(&mut config);
    // }
    config
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime
}

// javy_plugin!(config, modify_runtime);

// /// Evaluates QuickJS bytecode
// ///
// /// # Safety
// ///
// /// * `bytecode_ptr` must reference a valid array of unsigned bytes of `bytecode_len` length
// // This will be removed as soon as we stop emitting calls to it in dynamically
// // linked modules.
// #[export_name = "eval_bytecode"]
// pub unsafe extern "C" fn eval_bytecode(bytecode_ptr: *const u8, bytecode_len: usize) {
//     let bytecode = std::slice::from_raw_parts(bytecode_ptr, bytecode_len);
//     javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap();
//     javy_plugin_api::run_bytecode(bytecode, None);
// }
