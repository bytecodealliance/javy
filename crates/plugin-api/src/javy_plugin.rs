/// Provides default implementations for required methods a Javy plugin.
///
/// # Arguments
///
/// * `namespace` - The name of the module that will be used for function
///   imports in dynamically linked modules generated with this plugin.
/// * `component` - A struct.
/// * `config` - A function that will return a [`javy::Config`] to configure
///   the [`javy::Runtime`].
/// * `modify_runtime` - A function that can add modify the [`javy::Runtime`].
///   For example, by adding additional methods for use by JavaScript.
///
/// # Examples
///
/// ```ignore
/// use javy_plugin_api::{javy::{Config, Runtime}, javy_plugin};
///
/// struct Component;
///
/// javy_plugin!("my-import-namespace", Component, config, modify_runtime);
///
/// fn config() -> Config {
///     Config::default()
/// }
///
/// fn modify_runtime(runtime: Runtime) -> Runtime {
///     runtime
/// }
/// ```
#[macro_export]
macro_rules! javy_plugin {
    ($namespace:literal, $component:ident, $config:expr, $modify_runtime:expr) => {
        javy_plugin_api::import_namespace!($namespace);

        impl Guest for $component {
            fn compile_src(src: Vec<u8>) -> Result<Vec<u8>, String> {
                javy_plugin_api::compile_src(&src).map_err(|e| e.to_string())
            }

            fn initialize_runtime() -> () {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
            }

            fn invoke(bytecode: Vec<u8>, function: Option<String>) -> () {
                javy_plugin_api::invoke(&bytecode, function.as_deref()).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    std::process::abort();
                });
            }
        }
    };
}
