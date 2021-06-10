use quickjs_sys::*;
use std::{ffi::CString, os::raw::c_char};

#[derive(Debug, Copy, Clone)]
pub struct Context {
    pub raw: *mut JSContext,
    pub rt: *mut JSRuntime,
}

impl Context {
    pub fn new() -> Option<Self> {
        let rt = unsafe { JS_NewRuntime() };
        if rt.is_null() {
            return None;
        }

        let context = unsafe { JS_NewContext(rt) };
        if context.is_null() {
            // Free the runtime
            return None;
        }

        Some(Self {
            raw: context,
            rt,
        })
    }

    // By default this just evaluates the code
    // in the global scope
    pub fn eval(&self, bytes: &[u8], name: &str) {
        let input = make_cstring(bytes.to_vec());
        let script_name = make_cstring(name);

        unsafe {
            JS_Eval(
                self.raw,
                input.as_ptr(),
                (bytes.len() - 1) as _,
                script_name.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32
            );
        }
    }

    pub fn global(&self) -> JSValue {
        unsafe { JS_GetGlobalObject(self.raw) }
    }

    pub fn get_property(&self, name: &str, from: JSValue) -> JSValue {
        unsafe { JS_GetPropertyStr(self.raw, from, make_cstring(name).as_ptr()) }
    }

    pub fn call(&self, fun: JSValue, from: JSValue, args: &[JSValue]) -> JSValue {
        unsafe { JS_Call(self.raw, fun, from, args.len() as i32, args.as_ptr() as *mut u64) }
    }

    // TODO: Make this more generic, probably put it in a convert mod
    pub fn serialize_string(&self, val: &str) -> JSValue {
        unsafe { JS_NewStringLen(self.raw, val.as_ptr() as *const c_char, val.len() as _) }
    }

    pub fn deserialize_string(&self, val: JSValue) -> String {
        let ptr = unsafe { JS_ToCStringLen2(self.raw, std::ptr::null_mut(), val, 0) };
        let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
        cstr
            .to_str()
            .unwrap()
            .to_string()
    }

}

fn make_cstring(value: impl Into<Vec<u8>>) -> CString {
    CString::new(value).unwrap()
}
