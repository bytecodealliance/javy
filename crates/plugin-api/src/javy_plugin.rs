#[macro_export]
macro_rules! javy_plugin {
    ($component:ident, $invokable_trait:path, $config:expr, $modify_runtime:expr) => {
        impl crate::exports::bytecodealliance::javy_plugin::javy_plugin_exports::Guest
            for $component
        {
            fn compile_src(src: Vec<u8>) -> Vec<u8> {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
                javy_plugin_api::compile_src(&src)
            }

            fn initialize_runtime() -> () {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
            }
        }

        impl $invokable_trait for $component {
            fn invoke(bytecode: Vec<u8>, function: Option<String>) -> () {
                javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
                javy_plugin_api::invoke(&bytecode, function.as_deref())
            }
        }
    };
}
