use std::{
    alloc::{self, Layout},
    cell::OnceCell,
    process, ptr, slice,
};

static mut COMPILE_SRC_RET_AREA: [u32; 3] = [0; 3];

thread_local! {
    static BYTECODE: OnceCell<Vec<u8>> = const { OnceCell::new() };
}

// Unlike C's realloc, zero-length allocations need not have
// unique addresses, so a zero-length allocation may be passed
// in and also requested, but it's ok to return anything that's
// non-zero to indicate success.
const ZERO_SIZE_ALLOCATION_PTR: *mut u8 = 1 as _;

/// Allocates memory in instance.
///
/// 1. Allocate memory of new_size with alignment.
/// 2. If original_ptr != 0.  
///    a. copy min(new_size, original_size) bytes from original_ptr to new memory.  
///    b. de-allocate original_ptr.
/// 3. Return new memory ptr.
///
/// # Safety
///
/// * `original_ptr` must be 0 or a valid pointer.
/// * If `original_ptr` is not 0, it must be valid for reads of `original_size`
///   bytes.
/// * If `original_ptr` is not 0, it must be properly aligned.
/// * If `original_size` is not 0, it must match the `new_size` value provided
///   in the original `cabi_realloc` call that returned `original_ptr`.
#[export_name = "cabi_realloc"]
unsafe extern "C" fn cabi_realloc(
    original_ptr: *mut u8,
    original_size: usize,
    alignment: usize,
    new_size: usize,
) -> *mut std::ffi::c_void {
    assert!(new_size >= original_size);

    let new_mem = match new_size {
        0 => ZERO_SIZE_ALLOCATION_PTR,
        // this call to `alloc` is safe since `new_size` must be > 0
        _ => alloc::alloc(Layout::from_size_align(new_size, alignment).unwrap()),
    };

    if !original_ptr.is_null() && original_size != 0 {
        ptr::copy_nonoverlapping(original_ptr, new_mem, original_size);
        alloc::dealloc(
            original_ptr,
            Layout::from_size_align(original_size, alignment).unwrap(),
        );
    }
    new_mem as _
}

#[export_name = "compile-src"]
unsafe fn compile_src(src_ptr: *const u8, src_len: usize) -> *const u32 {
    let src = slice::from_raw_parts(src_ptr, src_len);
    let (res, bytes) = match crate::compile_src(src) {
        Ok(bytecode) => (0, bytecode),
        Err(err) => (1, err.to_string().into_bytes()),
    };
    let len = bytes.len();
    BYTECODE.with(|key| key.set(bytes)).unwrap();
    COMPILE_SRC_RET_AREA[0] = res;
    COMPILE_SRC_RET_AREA[1] = BYTECODE.with(|key| key.get().unwrap().as_ptr()) as u32;
    COMPILE_SRC_RET_AREA[2] = len as u32;
    COMPILE_SRC_RET_AREA.as_ptr()
}

#[export_name = "invoke"]
fn invoke(
    bytecode_ptr: *const u8,
    bytecode_len: usize,
    fn_name_discriminator: u32,
    fn_name_ptr: *const u8,
    fn_name_len: usize,
) {
    let bytecode = unsafe { slice::from_raw_parts(bytecode_ptr, bytecode_len) };
    let mut fn_name = None;
    if fn_name_discriminator != 0 {
        let fn_name_string =
            String::from_utf8_lossy(unsafe { slice::from_raw_parts(fn_name_ptr, fn_name_len) })
                .into_owned();
        fn_name = Some(fn_name_string);
    }
    crate::invoke(bytecode, fn_name.as_deref()).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::abort();
    });
}
