use quickjs_wasm_sys::{JS_NewRuntime, JS_NewContext};

#[no_mangle] 
fn main() {
    let runtime = unsafe { JS_NewRuntime() } ;
    if runtime.is_null() {
        panic!("Couldn't create JavaScript runtime");
    }

    let inner = unsafe { JS_NewContext(runtime) };
    if inner.is_null() {
        panic!("Couldn't create JavaScript context");
    }
}
