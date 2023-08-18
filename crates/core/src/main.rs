use javy::Runtime;
use once_cell::sync::OnceCell;
use std::io::{self, Read};
use std::slice;
use std::str;
use std::string::String;

mod execution;
mod runtime;

const FUNCTION_MODULE_NAME: &str = "function.mjs";

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = runtime::new_runtime().unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    let bytecode = runtime
        .context()
        .compile_module("function.mjs", &contents)
        .unwrap();

    unsafe {
        RUNTIME.set(runtime).unwrap();
        BYTECODE.set(bytecode).unwrap();
    }
}

fn main() {
    let bytecode = unsafe { BYTECODE.take().unwrap() };
    let runtime = unsafe { RUNTIME.take().unwrap() };
    execution::run_bytecode(&runtime, &bytecode);
}

// Removed in post-processing.
/// Evaluates QuickJS bytecode and invokes the exported JS function name.
///
/// # Safety
///
/// * `fn_name_ptr` must reference a UTF-8 string with `fn_name_size` byte
///   length.
#[export_name = "javy.invoke"]
pub unsafe extern "C" fn invoke(fn_name_ptr: *mut u8, fn_name_size: usize) {
    let js_fn_name = str::from_utf8_unchecked(slice::from_raw_parts(fn_name_ptr, fn_name_size));
    let runtime = unsafe { RUNTIME.take().unwrap() };
    execution::invoke_function(&runtime, FUNCTION_MODULE_NAME, js_fn_name);
}
