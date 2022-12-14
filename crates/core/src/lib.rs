mod engine;

use quickjs_wasm_rs::{Context, Value};

use once_cell::sync::OnceCell;
use std::alloc::{alloc, dealloc, Layout};
use std::io::{self, Read, Write};
use std::ptr::copy_nonoverlapping;
use std::slice;

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

    let global = context.global_object().unwrap();
    inject_javy_globals(&context, &global);

    context
        .eval_global(
            "text-encoding.js",
            include_str!("../prelude/text-encoding.js"),
        )
        .unwrap();
    context
        .eval_global("io.js", include_str!("../prelude/io.js"))
        .unwrap();
    JS_CONTEXT.set(context).unwrap();
}

#[export_name = "compile-bytecode"]
pub unsafe extern "C" fn compile_bytecode(
    contents_ptr: *mut u8,
    contents_len: *mut u32,
    bytecode_len: *mut u32,
) -> u32 {
    let contents_slice = std::slice::from_raw_parts_mut(contents_ptr, contents_len as usize);
    let contents = std::str::from_utf8_unchecked(contents_slice);
    let context = Context::default();
    let bytecode = context.compile_global(SCRIPT_NAME, contents).unwrap();
    *bytecode_len = bytecode.len() as u32;

    let vec = bytecode;
    let ret = vec.as_ptr() as u32;
    std::mem::forget(vec);
    ret
}

/// Evaluates the JS source code
///
/// # Safety
///
/// See safety for https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
#[export_name = "init-src"]
pub unsafe extern "C" fn init_src(js_str_ptr: *mut u8, js_str_len: usize) {
    let base64_bytecode =
        std::str::from_utf8_unchecked(slice::from_raw_parts(js_str_ptr, js_str_len));
    let bytecode = base64::decode(base64_bytecode).unwrap();
    let context = JS_CONTEXT.get().unwrap();
    let _ = context.eval_binary(&bytecode).unwrap();
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

fn js_args_to_io_writer(args: &[Value]) -> anyhow::Result<(Box<dyn Write>, &[u8])> {
    // TODO: Should throw an exception
    let [fd, data, offset, length, ..] = args else {
        anyhow::bail!("Invalid number of parameters");
    };

    let offset: usize = (offset.as_f64()?.floor() as u64).try_into()?;
    let length: usize = (length.as_f64()?.floor() as u64).try_into()?;

    let fd: Box<dyn Write> = match fd.try_as_integer()? {
        1 => Box::new(std::io::stdout()),
        2 => Box::new(std::io::stderr()),
        _ => anyhow::bail!("Only stdout and stderr are supported"),
    };

    if !data.is_array_buffer() {
        anyhow::bail!("Data needs to be an ArrayBuffer");
    }
    let data = data.as_bytes()?;
    Ok((fd, &data[offset..(offset + length)]))
}

fn js_args_to_io_reader(args: &[Value]) -> anyhow::Result<(Box<dyn Read>, &mut [u8])> {
    // TODO: Should throw an exception
    let [fd, data, offset, length, ..] = args else {
        anyhow::bail!("Invalid number of parameters");
    };

    let offset: usize = (offset.as_f64()?.floor() as u64).try_into()?;
    let length: usize = (length.as_f64()?.floor() as u64).try_into()?;

    let fd: Box<dyn Read> = match fd.try_as_integer()? {
        0 => Box::new(std::io::stdin()),
        _ => anyhow::bail!("Only stdin is supported"),
    };

    if !data.is_array_buffer() {
        anyhow::bail!("Data needs to be an ArrayBuffer");
    }
    let data = data.as_bytes_mut()?;
    Ok((fd, &mut data[offset..(offset + length)]))
}

fn inject_javy_globals(context: &Context, global: &Value) {
    global
        .set_property(
            "__javy_io_writeSync",
            context
                .wrap_callback(|ctx, _this_arg, args| {
                    let (mut fd, data) = js_args_to_io_writer(args)?;
                    let n = fd.write(data)?;
                    fd.flush()?;
                    ctx.value_from_i32(n.try_into()?)
                })
                .unwrap(),
        )
        .unwrap();

    global
        .set_property(
            "__javy_io_readSync",
            context
                .wrap_callback(|ctx, _this_arg, args| {
                    let (mut fd, data) = js_args_to_io_reader(args)?;
                    let n = fd.read(data)?;
                    ctx.value_from_i32(n.try_into()?)
                })
                .unwrap(),
        )
        .unwrap();
}
