use quickjs_sys as q;
use std::ffi::CStr;

static mut CTX: Option<*mut q::JSContext> = None;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let rt = q::JS_NewRuntime();
        CTX = Some(q::JS_NewContext(rt));
    }
}

#[export_name = "run"]
pub extern "C" fn run() -> u64 {
    #[cfg(not(feature = "wizer"))]
    init();

    let code_str = "1 + 1\0";
    let code = CStr::from_bytes_with_nul(code_str.as_bytes()).unwrap();
    let script = CStr::from_bytes_with_nul("script\0".as_bytes()).unwrap();

    unsafe {
        let value = q::JS_Eval(
            CTX.unwrap(),
            code.as_ptr(),
            (code_str.len() - 1) as _,
            script.as_ptr(),
            q::JS_EVAL_TYPE_GLOBAL as i32,
        );
        return value;
    }
}
