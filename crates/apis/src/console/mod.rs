use std::io::Write;

use anyhow::{Error, Result};
use javy::{
    hold, hold_and_release, print,
    quickjs::{prelude::MutFn, Context, Function, Object, Value},
    to_js_error, Args, Runtime,
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
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push(' ');
        }
        print(arg, &mut buf)?;
    }

    writeln!(stream, "{buf}")?;

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
    fn test_console_log() -> Result<()> {
        let mut stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();
        register_console(ctx, stream.clone(), stream.clone())?;

        ctx.with(|this| {
            this.eval("console.log(\"hello world\");")?;
            assert_eq!(b"hello world\n", stream.buffer.borrow().as_slice());
            stream.clear();

            this.eval("console.log(\"bonjour\", \"le\", \"monde\")")?;
            assert_eq!(b"bonjour le monde\n", stream.buffer.borrow().as_slice());

            stream.clear();

            this.eval("console.log(2.3, true, { foo: 'bar' }, null, undefined)")?;
            assert_eq!(
                b"2.3 true [object Object] null undefined\n",
                stream.buffer.borrow().as_slice()
            );

            Ok::<_, Error>(())
        })?;

        Ok(())
    }

    #[test]
    fn test_console_error() -> Result<()> {
        let mut stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();
        register_console(ctx, stream.clone(), stream.clone())?;

        ctx.with(|this| {
            this.eval("console.error(\"hello world\");")?;
            assert_eq!(b"hello world\n", stream.buffer.borrow().as_slice());

            stream.clear();

            this.eval("console.error(\"bonjour\", \"le\", \"monde\")")?;
            assert_eq!(b"bonjour le monde\n", stream.buffer.borrow().as_slice());

            stream.clear();

            this.eval("console.error(2.3, true, { foo: 'bar' }, null, undefined)")?;
            assert_eq!(
                b"2.3 true [object Object] null undefined\n",
                stream.buffer.borrow().as_slice()
            );
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
