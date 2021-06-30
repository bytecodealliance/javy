use quickjs_sys as q;

mod context;
mod engine;
mod input;
mod output;
mod value;

use context::*;


#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: Option<Context> = None;
static mut ENTRYPOINT: Option<(q::JSValue, q::JSValue)> = None;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let bytes = include_bytes!("index.js");
    unsafe {
        JS_CONTEXT = Some(Context::new().unwrap());
        let context = JS_CONTEXT.unwrap();

        let _ = context.compile(bytes, "script");
        let global = context.global();
        let shopify = context.get_property("Shopify", global);
        let main = context.get_property("main", shopify);

        ENTRYPOINT = Some((shopify, main));
    }

}

#[export_name = "shopify_main"]
pub extern "C" fn run() {
    #[cfg(not(feature = "wizer"))]
    init();

    unsafe {
        let context = JS_CONTEXT.unwrap();
        let (shopify, main) = ENTRYPOINT.unwrap();
        let input_bytes = engine::load();

        let serializer = input::prepare(&context, &input_bytes);
        let result = serializer.context.call(main, shopify, &[serializer.value]);

        if serializer.context.is_exception(result) {
            let ex = q::JS_GetException(context.raw);
            let exception = context.to_string(ex);
            println!("{:?}", exception);
        }

        let output = output::prepare(&serializer.context, result);
        engine::store(&output);
    }
}

