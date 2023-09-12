use javy::Runtime;
use once_cell::sync::OnceCell;
use std::slice;
use std::str;

mod execution;
mod runtime;

const FUNCTION_MODULE_NAME: &str = "function.mjs";

static mut COMPILE_SRC_RET_AREA: [u32; 2] = [0; 2];

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();

/// Used by Wizer to preinitialize the module
#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = runtime::new_runtime().unwrap();
    unsafe { RUNTIME.set(runtime).unwrap() };
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
    // Use fresh runtime to avoid depending on Wizened runtime
    let runtime = runtime::new_runtime().unwrap();
    let js_src = str::from_utf8(slice::from_raw_parts(js_src_ptr, js_src_len)).unwrap();
    let bytecode = runtime
        .context()
        .compile_module(FUNCTION_MODULE_NAME, js_src)
        .unwrap();
    let bytecode_len = bytecode.len();
    // We need the bytecode buffer to live longer than this function so it can be read from memory
    let bytecode_ptr = Box::leak(bytecode.into_boxed_slice()).as_ptr();
    COMPILE_SRC_RET_AREA[0] = bytecode_ptr as u32;
    COMPILE_SRC_RET_AREA[1] = bytecode_len.try_into().unwrap();
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
    execution::run_bytecode(runtime, bytecode);
}

/// Evaluates QuickJS bytecode and invokes the exported JS function name.
///
/// # Safety
///
/// * `bytecode_ptr` must reference a valid array of bytes of `bytecode_len`
///   length.
/// * `fn_name_ptr` must reference a UTF-8 string with `fn_name_len` byte
///   length.
#[export_name = "invoke"]
pub unsafe extern "C" fn invoke(
    bytecode_ptr: *const u8,
    bytecode_len: usize,
    fn_name_ptr: *const u8,
    fn_name_len: usize,
) {
    let runtime = RUNTIME.get().unwrap();
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    let fn_name = str::from_utf8_unchecked(slice::from_raw_parts(fn_name_ptr, fn_name_len));
    execution::run_bytecode(runtime, bytecode);
    execution::invoke_function(runtime, FUNCTION_MODULE_NAME, fn_name);
}
