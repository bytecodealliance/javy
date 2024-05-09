use std::io::Write;

use anyhow::{Error, Result};
use javy::{
    hold, hold_and_release,
    quickjs::{
        convert, prelude::MutFn, Context, FromJs, Function, Object, String as JSString, Value,
    },
    to_js_error, to_string_lossy, Args, Runtime,
};

use crate::{APIConfig, JSApiSet};

pub(super) use config::ConsoleConfig;
pub use config::LogStream;

mod config;

pub(super) struct Console {}

impl Console {
    pub(super) fn new() -> Self {
        Console {}
    }
}

impl JSApiSet for Console {
    fn register(&self, runtime: &Runtime, config: &APIConfig) -> Result<()> {
        register_console(
            runtime.context(),
            config.console.log_stream.to_stream(),
            config.console.error_stream.to_stream(),
        )
    }
}

fn register_console<T, U>(context: &Context, mut log_stream: T, mut error_stream: U) -> Result<()>
where
    T: Write + 'static,
    U: Write + 'static,
{
    context.with(|this| {
        let globals = this.globals();
        let console = Object::new(this.clone())?;

        console.set(
            "log",
            Function::new(
                this.clone(),
                MutFn::new(move |cx, args| {
                    let (cx, args) = hold_and_release!(cx, args);
                    log(hold!(cx.clone(), args), &mut log_stream).map_err(|e| to_js_error(cx, e))
                }),
            )?,
        )?;

        console.set(
            "error",
            Function::new(
                this.clone(),
                MutFn::new(move |cx, args| {
                    let (cx, args) = hold_and_release!(cx, args);
                    log(hold!(cx.clone(), args), &mut error_stream).map_err(|e| to_js_error(cx, e))
                }),
            )?,
        )?;

        globals.set("console", console)?;
        Ok::<_, Error>(())
    })?;
    Ok(())
}

fn log<'js, T: Write>(args: Args<'js>, stream: &mut T) -> Result<Value<'js>> {
    let (ctx, args) = args.release();
    for (i, arg) in args.iter().enumerate() {
        let stringified =
            <convert::Coerced<JSString>>::from_js(&ctx, arg.clone()).map(|string| {
                string
                    .to_string()
                    .unwrap_or_else(|e| to_string_lossy(&ctx, &string.0, e))
            })?;
        if i != 0 {
            write!(stream, " ")?;
        }
        write!(stream, "{stringified}")?;
    }
    writeln!(stream)?;

    Ok(Value::new_undefined(ctx.clone()))
}

#[cfg(test)]
mod tests {
    use anyhow::{Error, Result};
    use javy::{
        quickjs::{Object, Value},
        Runtime,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::{cmp, io};

    use crate::console::register_console;
    use crate::{APIConfig, JSApiSet};

    use super::Console;

    #[test]
    fn test_register() -> Result<()> {
        let runtime = Runtime::default();
        Console::new().register(&runtime, &APIConfig::default())?;
        runtime.context().with(|cx| {
            let console: Object<'_> = cx.globals().get("console")?;
            assert!(console.get::<&str, Value<'_>>("log").is_ok());
            assert!(console.get::<&str, Value<'_>>("error").is_ok());

            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_value_serialization() -> Result<()> {
        let mut stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();
        register_console(ctx, stream.clone(), stream.clone())?;

        ctx.with(|this| {
            macro_rules! test_console_log {
                ($js:expr, $expected:expr) => {{
                    this.eval($js)?;
                    assert_eq!(
                        $expected,
                        std::str::from_utf8(stream.buffer.borrow().as_slice()).unwrap()
                    );
                    stream.clear();
                }};
            }

            test_console_log!("console.log(\"hello world\");", "hello world\n");

            // Invalid UTF-16 surrogate pair
            test_console_log!("console.log(\"\\uD800\");", "�\n");

            test_console_log!(
                "console.log(function(){ return 1 })",
                "function(){ return 1 }\n"
            );

            test_console_log!(
                "console.log([1, \"two\", 3.42, null, 5])",
                "1,two,3.42,,5\n"
            );

            test_console_log!(
                "console.log(2.3, true, { foo: 'bar' }, null, undefined)",
                "2.3 true [object Object] null undefined\n"
            );

            test_console_log!(
                "console.log(new Date(0))",
                "Thu Jan 01 1970 00:00:00 GMT+0000\n"
            );

            test_console_log!("console.log(new ArrayBuffer())", "[object ArrayBuffer]\n");

            test_console_log!("console.log(NaN)", "NaN\n");

            test_console_log!("console.log(new Set())", "[object Set]\n");

            test_console_log!("console.log(new Map())", "[object Map]\n");

            test_console_log!(
                "function Foo(){}; console.log(new Foo())",
                "[object Object]\n"
            );

            Ok::<_, Error>(())
        })?;

        Ok(())
    }

    #[test]
    fn test_console_streams() -> Result<()> {
        let mut log_stream = SharedStream::default();
        let error_stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();
        register_console(ctx, log_stream.clone(), error_stream.clone())?;

        ctx.with(|this| {
            this.eval("console.log(\"hello world\");")?;
            assert_eq!(b"hello world\n", log_stream.buffer.borrow().as_slice());
            assert!(error_stream.buffer.borrow().is_empty());

            log_stream.clear();

            this.eval("console.error(\"hello world\");")?;
            assert_eq!(b"hello world\n", error_stream.buffer.borrow().as_slice());
            assert!(log_stream.buffer.borrow().is_empty());

            Ok::<_, Error>(())
        })?;

        Ok(())
    }

    #[derive(Clone)]
    struct SharedStream {
        buffer: Rc<RefCell<Vec<u8>>>,
        capacity: usize,
    }

    impl Default for SharedStream {
        fn default() -> Self {
            Self {
                buffer: Default::default(),
                capacity: usize::MAX,
            }
        }
    }

    impl SharedStream {
        fn clear(&mut self) {
            (*self.buffer).borrow_mut().clear();
        }
    }

    impl io::Write for SharedStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let available_capacity = self.capacity - (*self.buffer).borrow().len();
            let leftover = cmp::min(available_capacity, buf.len());
            (*self.buffer).borrow_mut().write(&buf[..leftover])
        }

        fn flush(&mut self) -> io::Result<()> {
            (*self.buffer).borrow_mut().flush()
        }
    }
}
