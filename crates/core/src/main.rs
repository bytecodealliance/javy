use quickjs_wasm_rs::{Context, Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read, Write};

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
            "text-encoding.js",
            include_str!("../prelude/text-encoding.js"),
        )
        .unwrap();

    let global = context.global_object().unwrap();
    inject_javy_globals(&context, &global);

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

fn inject_javy_globals(context: &Context, global: &Value) {
    global
        .set_property(
            "__javy_io_writeSync",
            context
                .wrap_callback(|ctx, this_arg, args| {
                    println!("Writing");
                    // if argc != 0 {
                    // return context.exception_value().unwrap().into();
                    // }

                    // let mut buffer: String = String::new();
                    // io::stdin().read_to_string(&mut buffer).unwrap();

                    // let val = context.value_from_str(&buffer).unwrap();
                    // val.into()
                    Ok(ctx.undefined_value().unwrap())
                })
                .unwrap(),
        )
        .unwrap();

    global
        .set_property(
            "__javy_io_readSync",
            context
                .wrap_callback(|ctx, this_arg, args| {
                    println!("Reading");
                    // if argc != 0 {
                    // return context.exception_value().unwrap().into();
                    // }

                    // let mut buffer: String = String::new();
                    // io::stdin().read_to_string(&mut buffer).unwrap();

                    // let val = context.value_from_str(&buffer).unwrap();
                    // val.into()
                    Ok(ctx.undefined_value().unwrap())
                })
                .unwrap(),
        )
        .unwrap();
}

fn main() {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    context.eval_global("function.mjs", &code).unwrap();
}
