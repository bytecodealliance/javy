use quickjs_sys::*;
use std::{io::Write, os::raw::c_int};

use super::context::Context;

pub fn register_globals<T>(ctx: &mut Context, log_stream: T)
where
    T: Write,
{
    let console_log_callback = ctx.new_callback(console_log_to(log_stream));
    let global_object = ctx.global();
    let console_object = ctx.new_object();
    ctx.set_property(console_object, "log", console_log_callback);
    ctx.set_property(global_object, "console", console_object);
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue
where
    T: Write,
{
    move |ctx: *mut JSContext, _this: JSValue, argc: c_int, argv: *mut JSValue, _magic: c_int| {
        let len: *mut size_t = &mut 0;
        for i in 0..argc {
            if i != 0 {
                write!(stream, " ").unwrap();
            }

            let str_ptr = unsafe { JS_ToCStringLen2(ctx, len, *argv.offset(i as isize), 0) };
            if str_ptr == std::ptr::null() {
                return unsafe { ext_js_exception };
            }

            let str_ptr = str_ptr as *const u8;
            let str_len = unsafe { *len as usize };
            let buffer = unsafe { std::slice::from_raw_parts(str_ptr, str_len) };

            stream.write(buffer).unwrap();
            unsafe { JS_FreeCString(ctx, str_ptr as *const i8) };
        }

        write!(stream, "\n").unwrap();
        unsafe { ext_js_undefined }
    }
}

#[cfg(test)]
mod tests {
    use super::register_globals;
    use crate::context::Context;

    #[test]
    fn test_console_log() {
        let mut stream: Vec<u8> = Vec::new();

        let mut ctx = Context::default();
        register_globals(&mut ctx, &mut stream);
        ctx.eval(b"console.log(\"hello world\");", "main");
        assert_eq!(b"hello world\n", &stream[..]);

        stream.clear();

        ctx.eval(b"console.log(\"bonjour\", \"le\", \"monde\")", "main");
        assert_eq!(b"bonjour le monde\n", &stream[..]);
    }
}
