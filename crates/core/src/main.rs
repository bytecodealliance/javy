use javy2::{ApiConfig, Runtime};
use once_cell::sync::OnceCell;
use quickjs_wasm_rs::JSContextRef;
use std::io::{self, Read, Stderr};
use std::string::String;

mod execution;

static mut RUNTIME: OnceCell<Runtime<Stderr, Stderr>> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = JSContextRef::default();
    let config = ApiConfig {
        log_stream: io::stderr(),
        error_stream: io::stderr(),
    };
    let mut rt = javy2::Runtime {
        context,
        api_config: config,
    };
    javy2::register_apis(&mut rt);

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    let bytecode = rt
        .context
        .compile_module("function.mjs", &contents)
        .unwrap();

    unsafe {
        RUNTIME.set(rt).unwrap();
        BYTECODE.set(bytecode).unwrap();
    }
}

fn main() {
    let bytecode = unsafe { BYTECODE.take().unwrap() };
    let rt = unsafe { RUNTIME.take().unwrap() };
    execution::run_bytecode(&rt.context, &bytecode).unwrap();
}
