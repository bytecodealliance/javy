use quickjs_wasm_rs::{Context, Value};

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

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

fn create_wasi_object(context: &Context) -> Value {
    let wasi_global = context.object_value().unwrap();

    unsafe {
        wasi_global
            .set_property(
                "writeStdout",
                context
                    .new_callback(|_ctx, _this, argc, argv, _magic| {
                        if argc != 1 {
                            return context.exception_value().unwrap().into();
                        }
                        let param = context.value_from_ptr(*argv.offset(0)).unwrap();
                        let str = param.as_str().unwrap();
                        io::stdout().write_all(str.as_bytes()).unwrap();
                        context.undefined_value().unwrap().into()
                    })
                    .unwrap(),
            )
            .unwrap();

        wasi_global
            .set_property(
                "readStdin",
                context
                    .new_callback(|_ctx, _this, argc, _argv, _magic| {
                        if argc != 0 {
                            return context.exception_value().unwrap().into();
                        }

                        let mut buffer: String = String::new();
                        io::stdin().read_to_string(&mut buffer).unwrap();

                        let val = context.value_from_str(&buffer).unwrap();
                        val.into()
                    })
                    .unwrap(),
            )
            .unwrap();
    }

    wasi_global
}

fn main() {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    let global = context.global_object().unwrap();
    global
        .set_property("WASI", create_wasi_object(&context))
        .unwrap();

    context.eval_global("function.mjs", &code).unwrap();
}
