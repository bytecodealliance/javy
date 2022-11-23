mod engine;

include!("walloc_bindings.rs");

use std::alloc::{GlobalAlloc, Layout};

use quickjs_wasm_rs::{json, Context, Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};

//
// Implementation of global allocator using walloc
//
struct WAllocator {}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: WAllocator = WAllocator {};

unsafe impl Sync for WAllocator {}

unsafe impl GlobalAlloc for WAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _lock = lock::lock();
        return wmalloc(layout.size(), layout.align());
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let _lock = lock::lock();
        wfree(_ptr);
    }
}

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());
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
        context
            .register_globals(io::stderr(), io::stderr())
            .unwrap();

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
        let input_bytes = engine::load().expect("Couldn't load input");

        let input_value = json::transcode_input(context, &input_bytes).unwrap();
        let output_value = main.call(shopify, &[input_value]);

        if output_value.is_err() {
            panic!("{}", output_value.unwrap_err().to_string());
        }

        let output = json::transcode_output(output_value.unwrap()).unwrap();
        engine::store(&output).expect("Couldn't store output");
    }
    unsafe {
        let _ = wmalloc(1024, 16);
    }
}

#[cfg(target_feature = "atomics")]
mod lock {
    use crate::sync::atomic::{AtomicI32, Ordering::SeqCst};

    static LOCKED: AtomicI32 = AtomicI32::new(0);

    pub struct DropLock;

    pub fn lock() -> DropLock {
        loop {
            if LOCKED.swap(1, SeqCst) == 0 {
                return DropLock;
            }
        }
    }

    impl Drop for DropLock {
        fn drop(&mut self) {
            let r = LOCKED.swap(0, SeqCst);
            debug_assert_eq!(r, 1);
        }
    }
}

#[cfg(not(target_feature = "atomics"))]
mod lock {
    #[inline]
    pub fn lock() {} // no atomics, no threads, that's easy!
}
