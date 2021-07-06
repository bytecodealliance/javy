use quickjs_sys::*;
use std::{ffi::{CString, c_void}, os::raw::{c_char, c_int}};

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

    pub fn compile(&self, bytes: &[u8], name: &str) -> JSValue {
        let input = make_cstring(bytes.to_vec());
        let script_name = make_cstring(name);

        unsafe {
            JS_Eval(
                self.raw,
                input.as_ptr(),
                (bytes.len() - 1) as _,
                script_name.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        }
    }

    pub fn eval_bytecode(&self, val: JSValue) -> JSValue {
        unsafe {
            JS_EvalFunction(self.raw, val)
        }
    }

    pub fn global(&self) -> JSValue {
        unsafe { JS_GetGlobalObject(self.raw) }
    }

    pub fn get_property(&self, name: &str, from: JSValue) -> JSValue {
        unsafe { JS_GetPropertyStr(self.raw, from, make_cstring(name).as_ptr()) }
    }

    pub fn define_property(&self, target: JSValue, key: &str, val: JSValue) {
        let key_name = make_cstring(key);
        unsafe { JS_DefinePropertyValueStr(self.raw, target, key_name.as_ptr(), val, JS_PROP_C_W_E as i32); }
    }

    pub fn define_property_raw(&self, target: JSValue, key: *const i8, val: JSValue) {
        unsafe { JS_DefinePropertyValueStr(self.raw, target, key, val, JS_PROP_C_W_E as i32) };
    }

    pub fn call(&self, fun: JSValue, from: JSValue, args: &[JSValue]) -> JSValue {
        unsafe { JS_Call(self.raw, fun, from, args.len() as i32, args.as_ptr() as *mut u64) }
    }

    pub fn serialize_array_buffer(&self, bytes: &[u8]) -> JSValue {
        unsafe { JS_NewArrayBuffer(self.raw, bytes.as_ptr() as *mut u8, bytes.len() as _, None, bytes.as_ptr() as *mut _, 0) }
    }

    pub fn serialize_string(&self, val: &str) -> JSValue {
        unsafe { JS_NewStringLen(self.raw, val.as_ptr() as *const c_char, val.len() as _) }
    }

    pub fn new_array(&self) -> JSValue {
        unsafe {
            JS_NewArray(self.raw)
        }
    }

    pub fn new_object(&self) -> JSValue {
        unsafe {
            JS_NewObject(self.raw)
        }
    }

    pub fn define_array_property(&self, target: JSValue, val: JSValue) {
        unsafe {
            let len = JS_GetPropertyStr(self.raw, target, make_cstring("length").as_ptr());
            JS_DefinePropertyValueUint32(self.raw, target, len as u32, val, JS_PROP_C_W_E as i32);
        }
    }

    pub fn parse_json(&self, json_string: &str) -> JSValue {
        let buf = json_string.as_ptr() as *const std::os::raw::c_char;
        unsafe { JS_ParseJSON(self.raw, buf, json_string.len() as _, make_cstring("input").as_ptr()) }
    }

    pub fn stringify_json(&self, obj: JSValue) -> JSValue {
        unsafe { JS_JSONStringify(self.raw, obj, JS_TAG_UNDEFINED as u64, JS_TAG_UNDEFINED as u64) }
    }

    pub fn get_tag(&self, val: JSValue) -> u64 {
        val >> 32
    }

    pub fn to_string(&self, val: JSValue) -> String {
        let string = unsafe { JS_ToString(self.raw, val) };
        self.deserialize_string(string)
    }

    pub fn is_exception(&self, val: JSValue) -> bool {
        self.get_tag(val) == JS_TAG_EXCEPTION as u64
    }

    pub fn is_undefined(&self, val: JSValue) -> bool {
        self.get_tag(val) == JS_TAG_UNDEFINED as u64
    }

    pub fn is_null(&self, val: JSValue) -> bool {
        self.get_tag(val) == JS_TAG_NULL as u64
    }

    pub fn is_obj(&self, val: JSValue) -> bool {
        self.get_tag(val) == JS_TAG_OBJECT as u64
    }

    pub fn as_c_str_ptr(&self, val: JSValue) -> *const i8 {
        unsafe { JS_ToCStringLen2(self.raw, std::ptr::null_mut(), val, 0) }
    }

    pub fn deserialize_string(&self, val: JSValue) -> String {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.as_c_str_ptr(val)) };
        cstr
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn create_callback<'a, F>(&self, callback: F) -> JSValue
    where F: Fn() -> JSValue + 'a
    {
        let f = move |_argc: c_int, _argv: *mut JSValue| -> JSValue {
            callback()
        };

        let (data, trampoline) = unsafe { build_trampoline(f) };
        unsafe {
            JS_NewCFunctionData(self.raw, trampoline, 0, 1, 1, data)
        }
    }
}

unsafe fn build_trampoline<F>(closure: F) -> (*mut JSValue, JSCFunctionData)
    where F: Fn(c_int, *mut JSValue) -> JSValue
    {
        unsafe extern "C" fn trampoline<F>(
            _ctx: *mut JSContext,
            _this: JSValue,
            argc: c_int,
            argv: *mut JSValue,
            _magic: c_int,
            data: *mut JSValue,
            ) -> JSValue
            where
                F: Fn(c_int, *mut JSValue) -> JSValue,
            {
                let closure_ptr = data;
                let closure: &mut F = &mut *(closure_ptr as *mut F);
                (*closure)(argc, argv)
            }

        let boxed = Box::new(closure);
        let value = &*boxed as *const F as *mut c_void as *const JSValue as *mut JSValue;
        (value, Some(trampoline::<F>))
    }

fn make_cstring(value: impl Into<Vec<u8>>) -> CString {
    CString::new(value).unwrap()
}
