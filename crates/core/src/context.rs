use quickjs_sys::*;
use std::{ffi::CString, os::raw::c_char, ptr};

#[derive(Debug, Copy, Clone)]
pub struct Context {
    pub raw: *mut JSContext,
    pub rt: *mut JSRuntime,
}

// TODO
// Extract the 'pure value' functions

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

    pub fn global(&self) -> JSValue {
        unsafe { JS_GetGlobalObject(self.raw) }
    }

    pub fn call(&self, fun: JSValue, from: JSValue, args: &[JSValue]) -> JSValue {
        unsafe { JS_Call(self.raw, fun, from, args.len() as i32, args.as_ptr() as *mut u64) }
    }

    pub unsafe fn new_float64(&self, val: f64) -> JSValue {
        JS_NewFloat64_Ext(self.raw, val)
    }

    pub unsafe fn is_float64(&self, val: JSValue) -> bool {
        JS_IsFloat64_Ext(self.get_tag(val) as i32) > 0
    }

    pub unsafe fn new_bool(&self, val: bool) -> JSValue {
        JS_NewBool_Ext(self.raw, val as i32)
    }

    pub fn new_string(&self, val: &str) -> JSValue {
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

    pub fn get_str_property(&self, name: &str, from: JSValue) -> JSValue {
        unsafe { JS_GetPropertyStr(self.raw, from, make_cstring(name).as_ptr()) }
    }

    pub fn set_str_property(&self, target: JSValue, key: &str, val: JSValue) {
        let key_name = make_cstring(key);
        unsafe { JS_DefinePropertyValueStr(self.raw, target, key_name.as_ptr(), val, JS_PROP_C_W_E as i32); }
    }

    pub fn set_property_raw(&self, target: JSValue, key: *const i8, val: JSValue) {
        unsafe { JS_DefinePropertyValueStr(self.raw, target, key, val, JS_PROP_C_W_E as i32) };
    }

    pub fn set_uint32_property(&self, target: JSValue, val: JSValue) {
        unsafe {
            let len = self.get_str_property("length", target);
            JS_DefinePropertyValueUint32(self.raw, target, len as u32, val, JS_PROP_C_W_E as i32);
        }
    }

    pub fn get_uint32_property(&self, target: JSValue, at: u32) -> JSValue {
        unsafe {
            JS_GetPropertyUint32(self.raw, target, at)
        }
    }

    pub fn get_own_properties(&self, obj: JSValue) -> (*mut JSPropertyEnum, i32) {
        let flags = (JS_GPN_STRING_MASK | JS_GPN_SYMBOL_MASK | JS_GPN_ENUM_ONLY) as i32;
        let mut properties: *mut JSPropertyEnum = ptr::null_mut();
        let mut count = 0;

        let result = unsafe { JS_GetOwnPropertyNames(self.raw, &mut properties, &mut count, obj, flags) };
        assert!(result == 0);

        (properties, count as i32)
    }

    pub fn get_internal_property(&self, obj: JSValue, key: JSAtom) -> JSValue {
        unsafe {
            JS_GetPropertyInternal(self.raw, obj, key, obj, 0)
        }
    }

    pub fn json_parse(&self, json_string: &str) -> JSValue {
        let buf = json_string.as_ptr() as *const std::os::raw::c_char;
        unsafe { JS_ParseJSON(self.raw, buf, json_string.len() as _, make_cstring("input").as_ptr()) }
    }

    pub fn json_stringify(&self, obj: JSValue) -> JSValue {
        unsafe { JS_JSONStringify(self.raw, obj, JS_TAG_UNDEFINED as u64, JS_TAG_UNDEFINED as u64) }
    }

    pub fn get_tag(&self, val: JSValue) -> u64 {
        val >> 32
    }

    pub fn to_string(&self, val: JSValue) -> String {
        let string = unsafe { JS_ToString(self.raw, val) };
        self.deserialize_string(string)
    }

    pub fn atom_to_string(&self, atom: JSAtom) -> JSValue {
        unsafe { JS_AtomToString(self.raw, atom) }
    }

    pub fn to_c_str_ptr(&self, val: JSValue) -> *const i8 {
        unsafe { JS_ToCStringLen2(self.raw, std::ptr::null_mut(), val, 0) }
    }

    pub fn deserialize_string(&self, val: JSValue) -> String {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.to_c_str_ptr(val)) };
        cstr
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn is_exception(&self, val: JSValue) -> bool {
        self.get_tag(val) == JS_TAG_EXCEPTION as u64
    }

    pub fn is_array(&self, val: JSValue) -> bool {
      unsafe { JS_IsArray(self.raw, val) > 0 }
    }

    // pub fn create_callback<'a, F>(&self, callback: F) -> JSValue
    // where F: Fn() -> JSValue + 'a
    // {
    //     let f = move |_argc: c_int, _argv: *mut JSValue| -> JSValue {
    //         callback()
    //     };

    //     let (data, trampoline) = unsafe { build_trampoline(f) };
    //     unsafe {
    //         JS_NewCFunctionData(self.raw, trampoline, 0, 1, 1, data)
    //     }
    // }
}

// unsafe fn build_trampoline<F>(closure: F) -> (*mut JSValue, JSCFunctionData)
//     where F: Fn(c_int, *mut JSValue) -> JSValue
//     {
//         unsafe extern "C" fn trampoline<F>(
//             _ctx: *mut JSContext,
//             _this: JSValue,
//             argc: c_int,
//             argv: *mut JSValue,
//             _magic: c_int,
//             data: *mut JSValue,
//             ) -> JSValue
//             where
//                 F: Fn(c_int, *mut JSValue) -> JSValue,
//             {
//                 let closure_ptr = data;
//                 let closure: &mut F = &mut *(closure_ptr as *mut F);
//                 (*closure)(argc, argv)
//             }

//         let boxed = Box::new(closure);
//         let value = &*boxed as *const F as *mut c_void as *const JSValue as *mut JSValue;
//         (value, Some(trampoline::<F>))
//     }

fn make_cstring(value: impl Into<Vec<u8>>) -> CString {
    CString::new(value).unwrap()
}
