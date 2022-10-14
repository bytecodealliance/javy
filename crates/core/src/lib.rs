mod engine;

use quickjs_wasm_rs::{json, Context};

use json::{transcode_input, transcode_output};
use once_cell::sync::OnceCell;
use std::alloc::{alloc, dealloc, Layout};
use std::io::{self};
use std::ptr::copy_nonoverlapping;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static SCRIPT_NAME: &str = "script.js";

// Unlike C's realloc, zero-length allocations need not have
// unique addresses, so a zero-length allocation may be passed
// in and also requested, but it's ok to return anything that's
// non-zero to indicate success.
const ZERO_SIZE_ALLOCATION_PTR: *mut u8 = 1 as _;

/// Sets up the JS context for the engine to be used to run JS code
///
/// # Safety
///
/// A OnceCell value is set in this function so this function must be called only once.
#[export_name = "init-engine"]
pub unsafe extern "C" fn init_engine() {
    let mut context = Context::default();
    context
        .register_globals(io::stderr(), io::stderr())
        .unwrap();
    JS_CONTEXT.set(context).unwrap();
}

#[export_name = "compile-bytecode"]
pub unsafe extern "C" fn compile_bytecode(contents_ptr: *mut u8, contents_len: *mut u32, bytecode_len: *mut u32) -> *mut u8 {
    let context = Context::default();
    // FIXME buffer appears to be getting corrupted
    // eprintln!("contents_ptr = {}, contents_len = {}", contents_ptr as usize, contents_len as usize);
    let contents_slice = std::slice::from_raw_parts_mut(contents_ptr, contents_len as usize);
    // eprintln!("buffer = {:?}", contents_slice);
    let contents = std::str::from_utf8_unchecked(contents_slice);
    eprintln!("before compile_global");
    let bytecode = context.compile_global("index.js", contents).unwrap();
    eprintln!("after compile_global");
    *bytecode_len = bytecode.len() as u32;

    let mut vec = vec![];
    eprintln!("before extend_from_slice");
    vec.extend_from_slice(bytecode);
    let bytecode = Box::new(vec);

    // let bytecode = Box::new(context.compile_global("index.js", contents).unwrap() as *mut &[u8]);
    // *bytecode_ptr = 
    Box::into_raw(bytecode) as *mut u8
}

/// Evaluates the JS source code
///
/// # Safety
///
/// See safety for https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
#[export_name = "init-src"]
pub unsafe extern "C" fn init_src(js_str_ptr: *mut u8, js_str_len: usize) {
    let bytecode = Vec::from_raw_parts(js_str_ptr, js_str_len, js_str_len);
    let context = JS_CONTEXT.get().unwrap();
    let _ = context.eval_binary(&bytecode).unwrap();
}

/// Executes the JS code.
/// func_obj_path is expected to be a dot spearate path to the main function.
///
/// # Safety
///
/// See safety for https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
#[export_name = "execute"]
pub unsafe extern "C" fn execute(
    func_obj_path_is_some: u32,
    func_obj_path_ptr: *mut u8,
    func_obj_path_len: usize,
) {
    let func_obj_path = match func_obj_path_is_some {
        0 => "Shopify.main".to_string(),
        _ => String::from_utf8(Vec::from_raw_parts(
            func_obj_path_ptr,
            func_obj_path_len,
            func_obj_path_len,
        ))
        .unwrap(),
    };

    assert!(!func_obj_path.is_empty());

    let context = JS_CONTEXT.get().unwrap();
    let (this, func) = func_obj_path.split('.').fold(
        (
            context.global_object().unwrap(),
            context.global_object().unwrap(),
        ),
        |(_this, func), obj| {
            let next = func.get_property(obj).unwrap();
            (func, next)
        },
    );

    let input_bytes = engine::load().expect("Couldn't load input");
    let input_value = transcode_input(context, &input_bytes).unwrap();
    let output_value = func.call(&this, &[input_value]);

    if output_value.is_err() {
        panic!("{}", output_value.unwrap_err().to_string());
    }
    let output = transcode_output(output_value.unwrap()).unwrap();
    engine::store(&output).expect("Couldn't store output");
}

/// 1. Allocate memory of new_size with alignment.
/// 2. If original_ptr != 0
///   a. copy min(new_size, original_size) bytes from original_ptr to new memory
///    b. de-allocate original_ptr
/// 3. return new memory ptr
/// https://doc.rust-lang.org/std/alloc/struct.Layout.html
/// https://doc.rust-lang.org/std/alloc/fn.alloc.html
///
/// # Safety
///
/// See the following APIs for safety
///
/// * https://doc.rust-lang.org/std/alloc/fn.alloc.html
/// * https://doc.rust-lang.org/core/intrinsics/fn.copy_nonoverlapping.html
/// * https://doc.rust-lang.org/std/alloc/fn.dealloc.html
#[export_name = "canonical_abi_realloc"]
pub unsafe extern "C" fn canonical_abi_realloc(
    original_ptr: *mut u8,
    original_size: usize,
    alignment: usize,
    new_size: usize,
) -> *mut std::ffi::c_void {
    assert!(new_size >= original_size);

    let new_mem = match new_size {
        0 => ZERO_SIZE_ALLOCATION_PTR,
        _ => alloc(Layout::from_size_align(new_size, alignment).unwrap()),
    };

    if !original_ptr.is_null() && original_size != 0 {
        copy_nonoverlapping(original_ptr, new_mem, original_size);
        canonical_abi_free(original_ptr, original_size, alignment);
    }
    new_mem as _
}
/// Frees memory
///
/// # Safety
///
/// See https://doc.rust-lang.org/std/alloc/fn.dealloc.html
#[export_name = "canonical_abi_free"]
pub unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, size: usize, alignment: usize) {
    if size > 0 {
        dealloc(ptr, Layout::from_size_align(size, alignment).unwrap())
    };
}
