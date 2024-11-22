//! Plugin used for testing. We need a plugin with slightly different behavior
//! to validate a plugin is actually used when it should be.

use javy_plugin_api::{import_namespace, Config};

import_namespace!("test_plugin");

#[export_name = "initialize_runtime"]
pub extern "C" fn initialize_runtime() {
    let config = Config::default();
    javy_plugin_api::initialize_runtime(config, |runtime| {
        runtime
            .context()
            .with(|ctx| ctx.globals().set("plugin", true))
            .unwrap();
        runtime
    })
    .unwrap();
}
