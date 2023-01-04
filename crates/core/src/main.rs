use once_cell::sync::OnceCell;
use quickjs_wasm_rs::Context;
use std::io::{self, Read};
use std::string::String;

mod globals;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();
static mut CODE: OnceCell<String> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = Context::default();
    globals::inject_javy_globals(&context, io::stderr(), io::stderr()).unwrap();

    context
        .eval_global(
            "text-encoding.js",
            include_str!("../prelude/text-encoding.js"),
        )
        .unwrap();

    context
        .eval_global("io.js", include_str!("../prelude/io.js"))
        .unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

fn main() {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    context.eval_global("function.mjs", &code).unwrap();
}
