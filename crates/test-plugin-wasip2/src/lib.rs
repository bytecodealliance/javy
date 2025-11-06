//! Plugin used for testing. We need a plugin with slightly different behavior
//! to validate a plugin is actually used when it should be.

use javy_plugin_api::{
    javy::{quickjs::prelude::Func, Runtime},
    javy_plugin, Config,
};

wit_bindgen::generate!({ world: "javy-test-plugin", generate_all });

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

javy_plugin!("test-plugin-wasip2", Component, config, modify_runtime);

export!(Component);
