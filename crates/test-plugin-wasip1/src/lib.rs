use javy_plugin_api::{
    import_namespace,
    javy::{quickjs::prelude::Func, Runtime},
    Config,
};

import_namespace!("test-plugin-wasip1");

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
            .set(
                "func",
                Func::from(|| {
                    unsafe { crate::imported_function() };
                }),
            )
            .unwrap();
    });
    runtime
}

#[export_name = "initialize-runtime"]
fn initialize_runtime() {
    javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap()
}
