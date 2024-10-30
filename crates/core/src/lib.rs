use anyhow::anyhow;
use javy::Config;
use javy::Runtime;
use namespace::import_namespace;
use once_cell::sync::OnceCell;
use shared_config::SharedConfig;
use std::io;
use std::io::Read;
use std::slice;
use std::str;

mod execution;
mod namespace;
mod shared_config;

const FUNCTION_MODULE_NAME: &str = "function.mjs";

static mut COMPILE_SRC_RET_AREA: [u32; 2] = [0; 2];

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();

import_namespace!("javy_quickjs_provider_v3");

/// Used by Wizer to preinitialize the module.
#[export_name = "initialize_runtime"]
pub extern "C" fn initialize_runtime() {
    // Read shared config JSON in from stdin.
    // Using stdin instead of an environment variable because the value set for
    // an environment variable will persist as the value set for that environment
    // variable in subsequent invocations so a different value can't be used to
    // initialize a runtime with a different configuration.
    let mut config = Config::default();
    // Preserve defaults that used to be passed from the Javy CLI.
    config
        .text_encoding(true)
        .redirect_stdout_to_stderr(true)
        .javy_stream_io(true)
        .simd_json_builtins(true)
        .javy_json(true);

    let mut config_bytes = vec![];
    let shared_config = match io::stdin().read_to_end(&mut config_bytes) {
        Ok(0) => None,
        Ok(_) => Some(SharedConfig::parse_from_json(&config_bytes).unwrap()),
        Err(e) => panic!("Error reading from stdin: {e}"),
    };
    if let Some(shared_config) = shared_config {
        shared_config.apply_to_config(&mut config);
    }

    let runtime = Runtime::new(config).unwrap();
    unsafe {
        RUNTIME.take(); // Allow re-initializing.
        RUNTIME
            .set(runtime)
            // `unwrap` requires error `T` to implement `Debug` but `set`
            // returns the `javy::Runtime` on error and `javy::Runtime` does not
            // implement `Debug`.
            .map_err(|_| anyhow!("Could not pre-initialize javy::Runtime"))
            .unwrap();
    };
}

/// Compiles JS source code to QuickJS bytecode.
///
/// Returns a pointer to a buffer containing a 32-bit pointer to the bytecode byte array and the
/// u32 length of the bytecode byte array.
///
/// # Arguments
///
/// * `js_src_ptr` - A pointer to the start of a byte array containing UTF-8 JS source code
/// * `js_src_len` - The length of the byte array containing JS source code
///
/// # Safety
///
/// * `js_src_ptr` must reference a valid array of unsigned bytes of `js_src_len` length
#[export_name = "compile_src"]
pub unsafe extern "C" fn compile_src(js_src_ptr: *const u8, js_src_len: usize) -> *const u32 {
    // Use initialized runtime when compiling because certain runtime
    // configurations can cause different bytecode to be emitted.
    //
    // For example, given the following JS:
    // ```
    // function foo() {
    //   "use math"
    //   1234 % 32
    // }
    // ```
    //
    // Setting `config.bignum_extension` to `true` will produce different
    // bytecode than if it were set to `false`.
    let runtime = unsafe { RUNTIME.get().unwrap() };
    let js_src = str::from_utf8(slice::from_raw_parts(js_src_ptr, js_src_len)).unwrap();

    let bytecode = runtime
        .compile_to_bytecode(FUNCTION_MODULE_NAME, js_src)
        .unwrap();

    // We need the bytecode buffer to live longer than this function so it can be read from memory
    let len = bytecode.len();
    let bytecode_ptr = Box::leak(bytecode.into_boxed_slice()).as_ptr();
    COMPILE_SRC_RET_AREA[0] = bytecode_ptr as u32;
    COMPILE_SRC_RET_AREA[1] = len.try_into().unwrap();
    COMPILE_SRC_RET_AREA.as_ptr()
}

/// Evaluates QuickJS bytecode
///
/// # Safety
///
/// * `bytecode_ptr` must reference a valid array of unsigned bytes of `bytecode_len` length
#[export_name = "eval_bytecode"]
pub unsafe extern "C" fn eval_bytecode(bytecode_ptr: *const u8, bytecode_len: usize) {
    let runtime = RUNTIME.get().unwrap();
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    execution::run_bytecode(runtime, bytecode, None);
}

/// Evaluates QuickJS bytecode and optionally invokes exported JS function with
/// name.
///
/// # Safety
///
/// * `bytecode_ptr` must reference a valid array of bytes of `bytecode_len`
///   length.
/// * If `fn_name_ptr` is not 0, it must reference a UTF-8 string with
///   `fn_name_len` byte length.
#[export_name = "invoke"]
pub unsafe extern "C" fn invoke(
    bytecode_ptr: *const u8,
    bytecode_len: usize,
    fn_name_ptr: *const u8,
    fn_name_len: usize,
) {
    let runtime = RUNTIME.get().unwrap();
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    let fn_name = if !fn_name_ptr.is_null() && fn_name_len != 0 {
        Some(str::from_utf8_unchecked(slice::from_raw_parts(
            fn_name_ptr,
            fn_name_len,
        )))
    } else {
        None
    };
    execution::run_bytecode(runtime, bytecode, fn_name);
}
