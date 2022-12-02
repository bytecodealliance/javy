mod engine;

use quickjs_wasm_rs::{Context, Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};

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
        context
            .eval_global(
                "text-encoding.js",
                include_str!("../prelude/text-encoding.js"),
            )
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

        let input_value = context.array_buffer_value(&input_bytes).unwrap();
        let output_value = main.call(shopify, &[input_value]);

        if output_value.is_err() {
            panic!("{}", output_value.unwrap_err().to_string());
        }

        let output_value = output_value.unwrap();
        if !output_value.is_array_buffer() {
            panic!(
                "Only ArrayBuffers are supported as return values, a different type was returned"
            );
        }

        let output = output_value.as_bytes().unwrap();
        engine::store(output).expect("Couldn't store output");
    }
}
