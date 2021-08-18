use quickjs_sys as q;

mod context;
mod engine;
mod input;
mod output;
mod serialization;

use context::*;

use std::{env, fs, path::PathBuf};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: Option<Context> = None;
static mut ENTRYPOINT: Option<(q::JSValue, q::JSValue)> = None;

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    // This could be problematic given that the allowed dirs should be known ahead-of-time
    // For now the workaround in the CLI is to set ~ as the allow dir
    let input = env::var("JAVY_INPUT").expect("Couldn't read JAVY_INPUT env var");
    let script_name = input.clone();
    let js: PathBuf = input.into();
    let bytes = fs::read(js).unwrap();
    unsafe {
        JS_CONTEXT = Some(Context::new().unwrap());
        let context = JS_CONTEXT.unwrap();

        let _ = context.compile(&bytes, &script_name);
        let global = context.global();
        let shopify = context.get_str_property("Shopify", global);
        let main = context.get_str_property("main", shopify);

        ENTRYPOINT = Some((shopify, main));
    }
}

// TODO
//
// Improve ergonomics around errors
// Improve ergonomics around exceptions
#[export_name = "shopify_main"]
pub extern "C" fn run() {
    unsafe {
        let context = JS_CONTEXT.unwrap();
        let (shopify, main) = ENTRYPOINT.unwrap();
        let input_bytes = engine::load();

        let serializer = input::prepare(&context, &input_bytes);
        let result = context.call(main, shopify, &[serializer.value]);

        if context.is_exception(result) {
            let ex = q::JS_GetException(context.raw);
            let exception = context.to_string(ex);
            println!("{:?}", exception);
        }

        let mut output = output::prepare(&context, result);
        engine::store(&mut output);
    }
}
