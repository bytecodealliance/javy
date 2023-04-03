use once_cell::sync::OnceCell;
use javy_embedded::Runtime;
use std::io::{self, Read};
use std::string::String;

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = Runtime::default().unwrap();
    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    let bytecode = runtime.compile_module("function.mjs", &contents).unwrap();

    unsafe {
        RUNTIME.set(runtime).unwrap();
        BYTECODE.set(bytecode).unwrap();
    }
}

fn main() {
    let bytecode = unsafe { BYTECODE.take().unwrap() };
    let runtime = unsafe { RUNTIME.take().unwrap() };
    runtime.run_bytecode(&bytecode).unwrap();
}
