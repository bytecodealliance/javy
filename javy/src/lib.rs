use quickjs_sys as q;

mod context;
mod sample;
mod engine;
mod input;
mod output;

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

#[export_name = "shopify_main"]
pub extern "C" fn run() {
    #[cfg(not(feature = "wizer"))]
    init();

    unsafe {
        let context = JS_CONTEXT.unwrap();
        let main_pair = ENTRYPOINT.unwrap();
        let input_bytes = engine::load(); // sample::DATA;

        let input_value = input::prepare(&context, &input_bytes);
        let result = context.call(main_pair.1, main_pair.0, &[input_value]);

        let tag = result >> 32;
        if tag == q::JS_TAG_EXCEPTION as u64 {
            let ex = q::JS_GetException(context.raw);
            let exception = context.deserialize_string(to_string(context.raw, ex));
            println!("{:?}", exception);
        }

        let output_bytes = output::prepare(&context, result);


        //output_bytes.len() as i32
        engine::store(&output_bytes);
    }
}

fn to_string(context: *mut q::JSContext, value: q::JSValue) -> q::JSValue {
    unsafe { q::JS_ToString(context, value) }
}

