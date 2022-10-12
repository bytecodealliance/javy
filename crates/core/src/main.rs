mod engine;

use quickjs_wasm_rs::{json, Context, Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read, Write};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();
static mut CODE: OnceCell<String> = OnceCell::new();

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let mut context = Context::default();
    context
        .register_globals(io::stderr(), io::stderr())
        .unwrap();

    context
        .eval_global(
            "prelude.js",
            r#"
            "#,
        )
        .unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

fn create_wasi_global(context: &Context) -> Value {
    let wasi_global = context.object_value().unwrap();

    unsafe {
        wasi_global.set_property(
            "writeStdout",
            context
                .new_callback(|_ctx, _this, _argc, _argv, _magic| {
                    io::stdout().write_all("hello".as_bytes()).unwrap();
                    context.null_value().unwrap().into()
                })
                .unwrap(),
        );
    }

    wasi_global
}

fn main() {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    let global = context.global_object().unwrap();
    global.set_property("WASI", create_wasi_global(&context));

    context.eval_global("function.mjs", &code).unwrap();

    // let output = json::transcode_output(output_value.unwrap()).unwrap();
    // engine::store(&output).expect("Couldn't store output");
}
