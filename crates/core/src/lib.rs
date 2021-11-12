mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::context::Context;
use once_cell::sync::OnceCell;
use std::io;
use transcode::{transcode_input, transcode_output};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static SCRIPT_NAME: &str = "script.js";

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let mut context = Context::default();
        context.register_globals(io::stdout()).unwrap();
        JS_CONTEXT.set(context).unwrap();
    }
}

#[export_name = "core_malloc"]
pub extern "C" fn exported_malloc(size: usize) -> *mut std::ffi::c_void {
    // Leak the vec<u8>, transfering ownership to the caller.
    // TODO: Consider not zeroing memory (with_capacity & set_len before into_raw_parts).
    Box::into_raw(vec![0u8; size].into_boxed_slice()) as _
}

#[export_name = "run_js_script"]
pub extern "C" fn run(ptr: *const u8, len: usize) {
    let (context, js_str) = unsafe {
        let js_str: &[u8] = std::slice::from_raw_parts(ptr as *const u8, len);
        let js_str = std::str::from_utf8_unchecked(js_str);

        (JS_CONTEXT.get().unwrap(), js_str)
    };
    let _ = context.eval_global(SCRIPT_NAME, js_str).unwrap();
    let global = context.global_object().unwrap();
    let shopify = global.get_property("Shopify").unwrap();
    let main = shopify.get_property("main").unwrap();

    let input_bytes = engine::load().expect("Couldn't load input");
    let input_value = transcode_input(context, &input_bytes).unwrap();
    let output_value = main.call(&shopify, &[input_value]);

    if output_value.is_err() {
        panic!("{}", output_value.unwrap_err().to_string());
    }

    let output = transcode_output(output_value.unwrap()).unwrap();
    engine::store(&output).expect("Couldn't store output");
}
