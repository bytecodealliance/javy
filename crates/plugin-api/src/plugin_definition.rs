// #[macro_export]
// macro_rules! javy_plugin {
//     ($config:ident, $modify_runtime:ident) => {
//         #[export_name = "initialize_runtime"]
//         pub extern "C" fn initialize_runtime() {
//             javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
//         }

//         #[no_mangle]
//         pub unsafe extern "C" fn compile_src(
//             js_src_ptr: *const u8,
//             js_src_len: usize,
//         ) -> *const u32 {
//             javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
//             javy_plugin_api::compile_src(js_src_ptr, js_src_len)
//         }

//         #[export_name = "invoke"]
//         pub unsafe extern "C" fn invoke(
//             bytecode_ptr: *const u8,
//             bytecode_len: usize,
//             fn_name_ptr: *const u8,
//             fn_name_len: usize,
//         ) {
//             javy_plugin_api::initialize_runtime($config, $modify_runtime).unwrap();
//             javy_plugin_api::invoke(bytecode_ptr, bytecode_len, fn_name_ptr, fn_name_len);
//         }
//     };
// }
