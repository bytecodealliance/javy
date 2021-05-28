use quickjs_sys as q;
use std::ffi::{CStr, CString};
use std::fmt;

static mut CTX: Option<*mut q::JSContext> = None;
static mut DATA: Option<&[u8]> = None;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let rt = q::JS_NewRuntime();
        CTX = Some(q::JS_NewContext(rt));

        let bytes = include_bytes!("fib.js");
        let cstr = CString::new(bytes.to_vec()).unwrap();
        let code = cstr.as_c_str();
        let name = CStr::from_bytes_with_nul("script\0".as_bytes()).unwrap();

        let value = q::JS_Eval(
            CTX.unwrap(),
            code.as_ptr(),
            (bytes.len() - 1) as _,
            name.as_ptr(),
            q::JS_EVAL_FLAG_COMPILE_ONLY as i32,
        );

        let mut len = 0;
        let raw = q::JS_WriteObject(
            CTX.unwrap(),
            &mut len,
            value,
            q::JS_WRITE_OBJ_BYTECODE as i32,
        );

        let slice = std::slice::from_raw_parts(raw, len as usize);
        DATA = Some(slice);
    }
}

#[export_name = "run"]
pub extern "C" fn run() -> i32 {
    #[cfg(not(feature = "wizer"))]
    init();

    unsafe {
        let bytecode = DATA.unwrap();
        let len = bytecode.len();
        let ctx = CTX.unwrap();
        let func = q::JS_ReadObject(ctx, bytecode.as_ptr(), len as _, q::JS_READ_OBJ_BYTECODE as i32);
        let result = q::JS_EvalFunction(ctx, func);

        let tag = result >> 32;
        if tag == q::JS_TAG_INT as u64 {
            return result as i32;
        }
        return tag as i32;
    }
}

#[derive(Debug)]
struct JSValue {
    u: JSValueUnion,
    tag: i64,
}

pub union JSValueUnion {
    int32: i32,
    float64: f64,
    ptr: *mut ::std::os::raw::c_void
}

impl fmt::Debug for JSValueUnion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JSValueUnion")
         .field("int32", unsafe { &self.int32 })
         .field("float64", unsafe { &self.float64 })
         .field("ptr", unsafe { &self.ptr})
         .finish()
    }
}
