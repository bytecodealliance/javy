use javy::quickjs::JSContextRef;
use once_cell::sync::OnceCell;
use std::io::{self, Read};
use std::string::String;

mod execution;
mod globals;

static mut CONTEXT: OnceCell<JSContextRef> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let context = JSContextRef::default();
    globals::inject_javy_globals(&context, io::stderr(), io::stderr()).unwrap();

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
