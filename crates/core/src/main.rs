use javy::Runtime;
use once_cell::sync::OnceCell;
use std::io::{self, Read};
use std::string::String;

mod execution;
mod runtime;

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = runtime::new_runtime().unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    let bytecode = runtime
        .context()
        .compile_module("function.mjs", &contents)
        .unwrap();

    unsafe {
        RUNTIME.set(runtime).unwrap();
        BYTECODE.set(bytecode).unwrap();
    }
}

fn main() {
    let bytecode = unsafe { BYTECODE.take().unwrap() };
    let runtime = unsafe { RUNTIME.take().unwrap() };
    match execution::run_bytecode(&runtime, &bytecode) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error while running JS: {}", e);
        }
    }
}
