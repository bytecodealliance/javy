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
        let mut len: size_t = 0;
        for i in 0..argc {
            if i != 0 {
                write!(stream, " ").unwrap();
            }

            let str_ptr = unsafe { JS_ToCStringLen2(ctx, &mut len, *argv.offset(i as isize), 0) };
            if str_ptr.is_null() {
                return unsafe { ext_js_exception };
            }

            let str_ptr = str_ptr as *const u8;
            let str_len = len as usize;
            let buffer = unsafe { std::slice::from_raw_parts(str_ptr, str_len) };

            stream.write_all(buffer).unwrap();
            unsafe { JS_FreeCString(ctx, str_ptr as *const i8) };
        }

        writeln!(stream,).unwrap();
        unsafe { ext_js_undefined }
    }
}

#[cfg(test)]
mod tests {
    use super::register_globals;
    use crate::context::Context;
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    #[derive(Default, Clone)]
    struct SharedStream(Rc<RefCell<Vec<u8>>>);

    impl SharedStream {
        fn clear(&mut self) {
            (*self.0).borrow_mut().clear();
        }
    }

    impl io::Write for SharedStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            (*self.0).borrow_mut().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            (*self.0).borrow_mut().flush()
        }
    }

    #[test]
    fn test_console_log() {
        let mut stream = SharedStream::default();

        let mut ctx = Context::default();
        register_globals(&mut ctx, stream.clone());

        ctx.eval(b"console.log(\"hello world\");", "main");
        assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

        stream.clear();

        ctx.eval(b"console.log(\"bonjour\", \"le\", \"monde\")", "main");
        assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());
    }
}
