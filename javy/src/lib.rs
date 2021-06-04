use quickjs_sys as q;
use std::ffi::{CStr, CString};

static mut CTX: Option<*mut q::JSContext> = None;
static mut FN: Option<q::JSValue> = None;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let rt = q::JS_NewRuntime();
        CTX = Some(q::JS_NewContext(rt));

        let bytes = include_bytes!("fib.js");
        let cstr = CString::new(bytes.to_vec()).unwrap();
        let code = cstr.as_c_str();
        let name = CStr::from_bytes_with_nul("script\0".as_bytes()).unwrap();
        let ctx = CTX.unwrap();

        q::JS_Eval(
            CTX.unwrap(),
            code.as_ptr(),
            (bytes.len() - 1) as _,
            name.as_ptr(),
            q::JS_EVAL_TYPE_GLOBAL as i32
        );

        let o = q::JS_GetGlobalObject(ctx);
        let f = q::JS_GetPropertyStr(ctx, o, CStr::from_bytes_with_nul("fib\0".as_bytes()).unwrap().as_ptr());
        FN = Some(f);
    }
}

#[export_name = "run"]
pub extern "C" fn run() -> i32 {
    #[cfg(not(feature = "wizer"))]
    init();

    unsafe {
        let ctx = CTX.unwrap();
        let f = FN.unwrap();
        let o = q::JS_GetGlobalObject(ctx);
        let args = [5 as u64];
        let result = q::JS_Call(ctx, f, o, 1, args.as_ptr() as *mut u64);

        let tag = result >> 32;
        if tag == q::JS_TAG_INT as u64 {
            return result as i32;
        }
        return -1;
    }
}
