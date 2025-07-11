//! Plugin used for testing. We need a plugin with slightly different behavior
//! to validate a plugin is actually used when it should be.

use javy_plugin_api::{
    import_namespace,
    javy::{quickjs::prelude::Func, Runtime},
    Config,
};

use crate::exports::bytecodealliance::javy_plugin::javy_plugin_exports::Guest;

wit_bindgen::generate!({ world: "javy-test-plugin", generate_all });

import_namespace!("test_plugin");

fn config() -> Config {
    Config::default()
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime.context().with(|ctx| {
        ctx.globals().set("plugin", true).unwrap();
        ctx.globals()
            .set(
                "func",
                Func::from(|| {
                    crate::imported_function();
                }),
            )
            .unwrap();
    });
    runtime
}

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
}

export!(Component);
