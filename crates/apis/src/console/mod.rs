use std::io::Write;

use anyhow::{Error, Result};
use javy::{
    quickjs::{
        prelude::{MutFn, Rest},
        Context, Ctx, Function, Object, Value,
    },
    Runtime,
};

use crate::{print, APIConfig, JSApiSet};

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

fn register_console<'js, T, U>(context: &Context, log_stream: T, error_stream: U) -> Result<()>
where
    T: Write,
    U: Write,
{
    context.with(|cx| {
        let globals = cx.globals();
        let console = Object::new(cx)?;
        console.set(
            "log",
            Function::new(
                cx,
                MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
                    log(cx, &args, &mut log_stream).unwrap()
                }),
            )?,
        )?;

        console.set(
            "error",
            Function::new(
                cx,
                MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
                    log(cx, &args, &mut error_stream).unwrap()
                }),
            )?,
        )?;

        globals.set("console", console)?;
        Ok::<_, Error>(())
    });
    Ok(())
}

fn log<'js, T: Write>(ctx: Ctx<'js>, args: &[Value<'js>], mut stream: T) -> Result<Value<'js>> {
    let mut buf = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push(' ');
        }
        print(arg, &mut buf)?;
    }

    writeln!(stream, "{buf}")?;

    Ok(Value::new_undefined(ctx))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use javy::Runtime;
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
        let console = runtime.context().global_object()?.get_property("console")?;
        assert!(console.get_property("log").is_ok());
        assert!(console.get_property("error").is_ok());
        Ok(())
    }

    #[test]
    fn test_console_log() -> Result<()> {
        let mut stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();
        register_console(ctx, stream.clone(), stream.clone())?;

        ctx.eval_global("main", "console.log(\"hello world\");")?;
        assert_eq!(b"hello world\n", stream.buffer.borrow().as_slice());

        stream.clear();

        ctx.eval_global("main", "console.log(\"bonjour\", \"le\", \"monde\")")?;
        assert_eq!(b"bonjour le monde\n", stream.buffer.borrow().as_slice());

        stream.clear();

        ctx.eval_global(
            "main",
            "console.log(2.3, true, { foo: 'bar' }, null, undefined)",
        )?;
        assert_eq!(
            b"2.3 true [object Object] null undefined\n",
            stream.buffer.borrow().as_slice()
        );
        Ok(())
    }

    #[test]
    fn test_console_error() -> Result<()> {
        let mut stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();
        register_console(ctx, stream.clone(), stream.clone())?;

        ctx.eval_global("main", "console.error(\"hello world\");")?;
        assert_eq!(b"hello world\n", stream.buffer.borrow().as_slice());

        stream.clear();

        ctx.eval_global("main", "console.error(\"bonjour\", \"le\", \"monde\")")?;
        assert_eq!(b"bonjour le monde\n", stream.buffer.borrow().as_slice());

        stream.clear();

        ctx.eval_global(
            "main",
            "console.error(2.3, true, { foo: 'bar' }, null, undefined)",
        )?;
        assert_eq!(
            b"2.3 true [object Object] null undefined\n",
            stream.buffer.borrow().as_slice()
        );
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
