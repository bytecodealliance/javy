mod engine;

use quickjs_wasm_rs::{messagepack, Context, Value, imports::hashing};

use once_cell::sync::OnceCell;
use std::io::{self, Read};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::ptr::copy_nonoverlapping;
use std::alloc::{alloc, dealloc, Layout};

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());
static SCRIPT_NAME: &str = "script.js";

const ZERO_SIZE_ALLOCATION_PTR: *mut u8 = 1 as _;

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let mut context = Context::default();
        context
            .register_globals(io::stderr(), io::stderr())
            .unwrap();

        hashing::add_to_context(&context).unwrap();


        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents).unwrap();

        let _ = context.eval_global(SCRIPT_NAME, &contents).unwrap();
        let global = context.global_object().unwrap();
        let shopify = global.get_property("Shopify").unwrap();
        let main = shopify.get_property("main").unwrap();

        JS_CONTEXT.set(context).unwrap();
        ENTRYPOINT.0.set(shopify).unwrap();
        ENTRYPOINT.1.set(main).unwrap();
    }
}

fn main() {
    unsafe {
        let context = JS_CONTEXT.get().unwrap();
        let shopify = ENTRYPOINT.0.get().unwrap();
        let main = ENTRYPOINT.1.get().unwrap();
        // let input_bytes = engine::load().expect("Couldn't load input");

        // let input_value = messagepack::transcode_input(context, &input_bytes).unwrap();
        let res = main.call(shopify, &[]).unwrap();
        println!("{:?}", res.as_str().unwrap());

        // if output_value.is_err() {
        //     panic!("{}", output_value.unwrap_err().to_string());
        // }

        // let output = messagepack::transcode_output(output_value.unwrap()).unwrap();
        // engine::store(&output).expect("Couldn't store output");
    }
}

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

#[export_name = "canonical_abi_free"]
pub unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, size: usize, alignment: usize) {
    if size > 0 {
        dealloc(ptr, Layout::from_size_align(size, alignment).unwrap())
    };
}
