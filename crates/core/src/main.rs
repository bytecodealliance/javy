use javy::Runtime;
use once_cell::sync::OnceCell;
use std::io::{self, Read};
use std::string::String;

mod execution;
mod runtime;

const FUNCTION_MODULE_NAME: &str = "function.mjs";

static mut RUNTIME: OnceCell<Runtime> = OnceCell::new();
static mut BYTECODE: OnceCell<Vec<u8>> = OnceCell::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = runtime::new_runtime().unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    let bytecode = runtime
        .context()
        .compile_module(FUNCTION_MODULE_NAME, &contents)
        .unwrap();

    unsafe {
        RUNTIME.set(runtime).unwrap();
        BYTECODE.set(bytecode).unwrap();
    }
}

fn main() {
    let runtime = unsafe { RUNTIME.take().unwrap() };
    invoke(&runtime, "foo");
}

fn invoke(runtime: &Runtime, function_name: &str) {
    let context = runtime.context();
    context
        .eval_module(
            "runtime.mjs",
            &format!(
                "import {{ {function_name} }} from '{FUNCTION_MODULE_NAME}'; {function_name}();"
            ),
        )
        .unwrap();
}
