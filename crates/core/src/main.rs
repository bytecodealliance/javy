use javy2::ApiConfig;
use once_cell::sync::OnceCell;
use quickjs_wasm_rs::JSContextRef;
use std::io::{self, Read};
use std::string::String;

mod execution;

static mut CONTEXT: OnceCell<JSContextRef> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = JSContextRef::default();
    let rt = javy2::Runtime { context: &context };
    let mut config = ApiConfig {
        log_stream: io::stderr(),
        error_stream: io::stderr(),
    };
    javy2::register_apis(&rt, &mut config);

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    let bytecode = context.compile_module("function.mjs", &contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        BYTECODE.set(bytecode).unwrap();
    }
}

fn main() {
    let bytecode = unsafe { BYTECODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };
    execution::run_bytecode(&context, &bytecode).unwrap();
}
