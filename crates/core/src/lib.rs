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

/// Compiles JS source code to QuickJS bytecode.
///
/// Returns a pointer to a byte array containing the bytecode.
///
/// # Arguments
///
/// * `js_src_ptr` - A pointer to the start of a byte array containing UTF-8 JS source code
/// * `js_src_len` - The length of the byte array containing JS source code
/// * `bytecode_len_ptr` - A pointer to where to write the length of the returned bytecode byte array
///
/// # Safety
///
/// * `js_src_ptr` must reference a valid array of unsigned bytes of `js_src_len` length
/// * `bytecode_len` must be a pointer allocated by `canonical_abi_realloc` with an `alignment` of
///   4 and a `new_size` of 1
#[export_name = "compile_src"]
pub unsafe extern "C" fn compile_src(
    js_src_ptr: *const u8,
    js_src_len: usize,
    bytecode_len_ptr: *mut u32,
) -> *const u8 {
    let context = Context::default();
    let js_src = str::from_utf8(slice::from_raw_parts(js_src_ptr, js_src_len)).unwrap();
    let bytecode = context.compile_global("function.mjs", js_src).unwrap();
    *bytecode_len_ptr = bytecode.len().try_into().unwrap();
    // We need the bytecode buffer to live longer than this function so it can be read from memory
    Box::leak(bytecode.into_boxed_slice()).as_ptr()
}

/// Evaluates QuickJS bytecode
///
/// # Safety
///
/// * `bytecode_ptr` must reference a valid array of unsigned bytes of `bytecode_len` length
#[export_name = "eval_bytecode"]
pub unsafe extern "C" fn eval_bytecode(bytecode_ptr: *const u8, bytecode_len: usize) {
    let context = Context::default();
    globals::inject_javy_globals(&context, io::stderr(), io::stderr()).unwrap();
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
