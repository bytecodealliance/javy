use quickjs_sys as q;
use std::ffi::CString;

static mut JS_CONTEXT: Option<Context> = None;
static mut ENTRYPOINT: Option<q::JSValue> = None;

#[derive(Debug, Copy, Clone)]
struct Context {
    raw: *mut q::JSContext,
    rt: *mut q::JSRuntime,
}

impl Context {
    fn new() -> Option<Self> {
        let rt = unsafe { q::JS_NewRuntime() };
        if rt.is_null() {
            return None;
        }

        let context = unsafe { q::JS_NewContext(rt) };
        if context.is_null() {
            // Free the runtime
            return None;
        }

        Some(Self {
            raw: context,
            rt
        })
    }
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let bytes = include_bytes!("fib.js");
    let code = make_cstring(bytes.to_vec());

    unsafe {
        JS_CONTEXT = Some(Context::new().unwrap());
        let context = JS_CONTEXT.unwrap();

        q::JS_Eval(
            context.raw,
            code.as_ptr(),
            (bytes.len() - 1) as _,
            make_cstring("script").as_ptr(),
            q::JS_EVAL_TYPE_GLOBAL as i32
        );

        let global = q::JS_GetGlobalObject(context.raw);
        let entry = q::JS_GetPropertyStr(context.raw, global, make_cstring("main").as_ptr());
        ENTRYPOINT = Some(entry);
    }

}

#[export_name = "run"]
pub extern "C" fn run() -> i32 {
    #[cfg(not(feature = "wizer"))]
    init();

    unsafe {
        let context = JS_CONTEXT.unwrap();
        let main = ENTRYPOINT.unwrap();

        let global = q::JS_GetGlobalObject(context.raw);

        let encoded = rmp_serde::to_vec(&[1, 2, 3, 4, 5, 6, 7]).unwrap();
        let arg = serialize_byte_array(&context, &encoded);
        // TODO: Remove hardcoded arg count
        let result = q::JS_Call(context.raw, main, global, 1 as i32, [arg].as_ptr() as *mut u64);
        let values = deserialize_byte_array(&context, &result);

        values.len() as i32
    }
}

fn serialize_byte_array(context: &Context, bytes: &Vec<u8>) -> q::JSValue {
    let array = unsafe { q::JS_NewArray(context.raw) };

    for (idx, value) in bytes.into_iter().enumerate() {
        unsafe {
            q::JS_DefinePropertyValueUint32(context.raw, array, idx as u32, *value as u64, q::JS_PROP_C_W_E as i32)
        };
    }

    array
}

// The deserialization can fail in several ways. Those ways are not handled here
// for the sake of prototyping.
// 1. `length` might not exist in the array
// 2. Index of out bounds
// 3. Value might not be an integer (needs to be an integer, u8 more specifically)

fn deserialize_byte_array(ctx: &Context, val: &q::JSValue) -> Vec<u8> {
    let is_array = unsafe { q::JS_IsArray(ctx.raw, *val) };
    assert!(is_array > 0, "Expected array");
    let len = unsafe { q::JS_GetPropertyStr(ctx.raw, *val, make_cstring("length").as_ptr()) };
    let mut values = Vec::new();
    for index in 0..(len as usize) {
        let raw_value = unsafe { q::JS_GetPropertyUint32(ctx.raw, *val, index as u32) };
        values.push(raw_value as u8);
    }
    values
}

fn make_cstring(value: impl Into<Vec<u8>>) -> CString {
    CString::new(value).unwrap()
}

