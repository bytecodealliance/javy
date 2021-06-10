use quickjs_sys as q;

mod context;
mod sample;

use context::*;

static mut JS_CONTEXT: Option<Context> = None;
static mut ENTRYPOINT: Option<(q::JSValue, q::JSValue)> = None;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let bytes = include_bytes!("index.js");
    unsafe {
        JS_CONTEXT = Some(Context::new().unwrap());
        let context = JS_CONTEXT.unwrap();

        context.eval(bytes, "script");
        let global = context.global();
        let script = context.get_property("Shopify", global);
        let main = context.get_property("main", script);

        ENTRYPOINT = Some((script, main));
    }

}

#[export_name = "run"]
pub extern "C" fn run() -> i32 {
    #[cfg(not(feature = "wizer"))]
    init();

    unsafe {
        let context = JS_CONTEXT.unwrap();
        let main_pair = ENTRYPOINT.unwrap();

        let value: rmpv::Value = rmp_serde::from_slice(sample::DATA).unwrap();
        let json = serde_json::to_string(&value).unwrap();
        let arg = context.serialize_string(&json);

        let result = context.call(main_pair.1, main_pair.0, &[arg]);

        let tag = result >> 32;
        if tag == q::JS_TAG_EXCEPTION as u64 {
            let ex = q::JS_GetException(context.raw);
            let exception = context.deserialize_string(to_string(context.raw, ex));
            println!("{:?}", exception);
        }

        let response: serde_json::Value = serde_json::from_str(&context.deserialize_string(result)).unwrap();
        let bytes = rmp_serde::to_vec(&response).unwrap();

        return bytes.len() as i32;
    }
}



fn to_string(context: *mut q::JSContext, value: q::JSValue) -> q::JSValue {
    unsafe { q::JS_ToString(context, value) }
}
