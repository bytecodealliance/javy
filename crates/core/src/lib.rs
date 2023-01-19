use once_cell::sync::OnceCell;
use quickjs_wasm_rs::Context;
use std::alloc::{alloc, dealloc, Layout};
use std::io;
use std::ptr::copy_nonoverlapping;
use std::slice;
use std::str;

mod globals;

// Unlike C's realloc, zero-length allocations need not have
// unique addresses, so a zero-length allocation may be passed
// in and also requested, but it's ok to return anything that's
// non-zero to indicate success.
const ZERO_SIZE_ALLOCATION_PTR: *mut u8 = 1 as _;

static mut COMPILE_SRC_RET_AREA: [u32; 2] = [0; 2];

static mut CONTEXT: OnceCell<Context> = OnceCell::new();

/// Used by Wizer to preinitialize the module
#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = Context::default();
    globals::inject_javy_globals(&context, io::stderr(), io::stderr()).unwrap();
    unsafe { CONTEXT.set(context).unwrap() };
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
    // Use fresh context to avoid depending on Wizened context
    let context = Context::default();
    let js_src = str::from_utf8(slice::from_raw_parts(js_src_ptr, js_src_len)).unwrap();
    let bytecode = context.compile_global("function.mjs", js_src).unwrap();
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
    let context = CONTEXT.get().unwrap();
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    context.eval_binary(bytecode).unwrap();
}

/// 1. Allocate memory of new_size with alignment.
/// 2. If original_ptr != 0
///   a. copy min(new_size, original_size) bytes from original_ptr to new memory
///   b. de-allocate original_ptr
/// 3. return new memory ptr
///
/// # Safety
///
/// * `original_ptr` must be 0 or a valid pointer
/// * if `original_ptr` is not 0, it must be valid for reads of `original_size`
///   bytes
/// * if `original_ptr` is not 0, it must be properly aligned
/// * if `original_size` is not 0, it must match the `new_size` value provided
///   in the original `canonical_abi_realloc` call that returned `original_ptr`
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
        // this call to `alloc` is safe since `new_size` must be > 0
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
/// * `ptr` must denote a block of memory allocated by `canonical_abi_realloc`
/// * `size` and `alignment` must match the values provided in the original
///   `canonical_abi_realloc` call that returned `ptr`
#[export_name = "canonical_abi_free"]
pub unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, size: usize, alignment: usize) {
    if size > 0 {
        dealloc(ptr, Layout::from_size_align(size, alignment).unwrap())
    };
}
