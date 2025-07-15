#[macro_export]
macro_rules! javy_plugin {
    ($namespace:literal, $component:ident, $config:expr, $modify_runtime:expr) => {
        javy_plugin_api::import_namespace!($namespace);

        impl Guest for $component {
            fn compile_src(src: Vec<u8>) -> Vec<u8> {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
                javy_plugin_api::compile_src(&src)
            }

            fn initialize_runtime() -> () {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
            }

            fn invoke(bytecode: Vec<u8>, function: Option<String>) -> () {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
                javy_plugin_api::invoke(&bytecode, function.as_deref())
            }
        }
    };
}
