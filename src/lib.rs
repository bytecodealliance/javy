use libquickjs::{self, bindings};

static mut CTX: Option<*mut libquickjs::bindings::JSContext> = None;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let rt = bindings::JS_NewRuntime();
        CTX = Some(bindings::JS_NewContext(rt));
    }
}

#[export_name = "run"]
pub extern "C" fn run() -> i32 {
    #[cfg(not(wizer))]
    init();

    return 0;
}


