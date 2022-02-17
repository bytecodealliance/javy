use quickjs_wasm_sys::{JS_NewRuntime, JS_NewContext, JSContext, JS_Eval, JS_EVAL_TYPE_GLOBAL, JS_TAG_EXCEPTION};
use std::{ffi::CString, convert::TryInto};
use once_cell::sync::OnceCell;

static mut JS_CONTEXT: OnceCell<*mut JSContext> = OnceCell::new();
static mut SCRIPT: OnceCell<CString> = OnceCell::new();
static mut SCRIPT_LEN: OnceCell<u32> = OnceCell::new();
static mut SCRIPT_NAME: OnceCell<CString> = OnceCell::new();


#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let runtime = unsafe { JS_NewRuntime() } ;
    if runtime.is_null() {
        panic!("Couldn't create JavaScript runtime");
    }

    let inner = unsafe { JS_NewContext(runtime) };
    if inner.is_null() {
        panic!("Couldn't create JavaScript context");
    }

    unsafe { JS_CONTEXT.set(inner).unwrap(); }

    let contents = std::env::var("SCRIPT").unwrap();
    let len = contents.len() - 1;

    unsafe {
        SCRIPT.set(CString::new(contents).unwrap()).unwrap();
        SCRIPT_NAME.set(CString::new("script.js").unwrap()).unwrap();
        SCRIPT_LEN.set(len.try_into().unwrap()).unwrap();
    }
}

#[no_mangle]
fn main() {
    unsafe {
        let contents = SCRIPT.get().unwrap();
        let name = SCRIPT_NAME.get().unwrap();
        let len = SCRIPT_LEN.get().unwrap() - 1;
        let context = JS_CONTEXT.get().unwrap();

        let v = JS_Eval(
            *context,
            contents.as_ptr(),
            len as _,
            name.as_ptr(),
            JS_EVAL_TYPE_GLOBAL as i32
        );
         
        let tag = (v >> 32) as i32;

        if tag == JS_TAG_EXCEPTION {
            panic!("Script returned an exception");
        }
    }
}
