//! Plugin used for testing. We need a plugin with slightly different behavior
//! to validate a plugin is actually used when it should be.

use javy_plugin_api::{
    import_namespace,
    javy::{quickjs::prelude::Func, Runtime},
    javy_plugin, Config,
};

import_namespace!("test_plugin");

#[link(wasm_import_module = "some_host")]
extern "C" {
    fn imported_function();
}

fn config() -> Config {
    Config::default()
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime.context().with(|ctx| {
        ctx.globals().set("plugin", true).unwrap();
        ctx.globals()
            .set("func", Func::from(|| unsafe { imported_function() }))
            .unwrap();
    });
    runtime
}

javy_plugin!(config, modify_runtime);
