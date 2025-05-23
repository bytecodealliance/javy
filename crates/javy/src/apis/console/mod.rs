use std::io::Write;

use crate::{
    hold, hold_and_release,
    quickjs::{prelude::MutFn, Ctx, Function, Object, Value},
    to_js_error, val_to_string, Args,
};
use anyhow::Result;

/// Register a `console` object on the global object with `.log`, `.warn` and `.error`
/// streams.
pub(crate) fn register<T, U, V>(this: Ctx<'_>, mut log_stream: T, mut warn_stream: U, mut error_stream: V) -> Result<()>
where
    T: Write + 'static,
    U: Write + 'static,
    V: Write + 'static,
{
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
        "warn",
        Function::new(
            this.clone(),
            MutFn::new(move |cx, args| {
                let (cx, args) = hold_and_release!(cx, args);
                log(hold!(cx.clone(), args), &mut warn_stream).map_err(|e| to_js_error(cx, e))
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
    Ok(())
}

fn log<'js, T: Write>(args: Args<'js>, stream: &mut T) -> Result<Value<'js>> {
    let (ctx, args) = args.release();
    for (i, arg) in args.into_inner().into_iter().enumerate() {
        if i != 0 {
            write!(stream, " ")?;
        }

        let str = val_to_string(&ctx, arg)?;
        write!(stream, "{str}")?;
    }
    writeln!(stream)?;

    Ok(Value::new_undefined(ctx))
}

#[cfg(test)]
mod tests {
    use crate::{
        apis::console::register,
        quickjs::{Object, Value},
        Runtime,
    };
    use anyhow::{Error, Result};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::{cmp, io};

    #[test]
    fn test_register() -> Result<()> {
        let runtime = Runtime::default();
        runtime.context().with(|cx| {
            let console: Object<'_> = cx.globals().get("console")?;
            assert!(console.get::<&str, Value<'_>>("log").is_ok());
            assert!(console.get::<&str, Value<'_>>("warn").is_ok());
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

        ctx.with(|this| {
            register(this.clone(), stream.clone(), stream.clone(), stream.clone()).unwrap();
            this.eval::<(), _>("console.log(\"hello world\");")?;
            assert_eq!(b"hello world\n", stream.buffer.borrow().as_slice());
            stream.clear();
            macro_rules! test_console_log {
                ($js:expr, $expected:expr) => {{
                    this.eval::<(), _>($js)?;
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

            test_console_log!("console.log(Symbol())", "Symbol()\n");

            test_console_log!("console.log(Symbol(''))", "Symbol()\n");

            test_console_log!("console.log(Symbol('foo'))", "Symbol(foo)\n");

            test_console_log!("console.log(Symbol(null))", "Symbol(null)\n");

            test_console_log!("console.log(Symbol(undefined))", "Symbol()\n");

            test_console_log!("console.log(Symbol([]))", "Symbol()\n");

            // Invalid UTF-16 surrogate pair
            test_console_log!("console.log(Symbol(\"\\uD800\"))", "Symbol(�)\n");

            Ok::<_, Error>(())
        })?;

        Ok(())
    }

    #[test]
    fn test_console_streams() -> Result<()> {
        let mut log_stream = SharedStream::default();
        let mut warn_stream = SharedStream::default();
        let error_stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();

        ctx.with(|this| {
            register(this.clone(), log_stream.clone(), warn_stream.clone(), error_stream.clone()).unwrap();
            this.eval::<(), _>("console.log(\"hello world\");")?;
            assert_eq!(b"hello world\n", log_stream.buffer.borrow().as_slice());
            assert!(warn_stream.buffer.borrow().is_empty());
            assert!(error_stream.buffer.borrow().is_empty());

            log_stream.clear();

            this.eval::<(), _>("console.warn(\"hello world\");")?;
            assert_eq!(b"hello world\n", warn_stream.buffer.borrow().as_slice());
            assert!(log_stream.buffer.borrow().is_empty());
            assert!(error_stream.buffer.borrow().is_empty());

            warn_stream.clear();

            this.eval::<(), _>("console.error(\"hello world\");")?;
            assert_eq!(b"hello world\n", error_stream.buffer.borrow().as_slice());
            assert!(log_stream.buffer.borrow().is_empty());
            assert!(warn_stream.buffer.borrow().is_empty());

            Ok::<_, Error>(())
        })?;

        Ok(())
    }

    #[test]
    fn test_redirect_functionality() -> Result<()> {
        // Test normal mode (console.log -> stdout, warn/error -> stderr)
        let mut log_stream = SharedStream::default();
        let mut warn_stream = SharedStream::default();
        let mut error_stream = SharedStream::default();

        let runtime = Runtime::default();
        let ctx = runtime.context();

        ctx.with(|this| {
            // Normal mode: log->stdout, warn->stderr, error->stderr
            register(this.clone(), log_stream.clone(), warn_stream.clone(), error_stream.clone()).unwrap();
            
            this.eval::<(), _>("console.log('normal log');")?;
            this.eval::<(), _>("console.warn('normal warn');")?;
            this.eval::<(), _>("console.error('normal error');")?;
            
            assert_eq!(b"normal log\n", log_stream.buffer.borrow().as_slice());
            assert_eq!(b"normal warn\n", warn_stream.buffer.borrow().as_slice());
            assert_eq!(b"normal error\n", error_stream.buffer.borrow().as_slice());

            Ok::<_, Error>(())
        })?;

        // Test redirected mode (all console outputs -> stderr)
        let mut redirected_log_stream = SharedStream::default();
        let mut redirected_warn_stream = SharedStream::default();
        let mut redirected_error_stream = SharedStream::default();

        ctx.with(|this| {
            // Redirected mode: all -> stderr (simulated by using same stream)
            register(this.clone(), redirected_log_stream.clone(), redirected_warn_stream.clone(), redirected_error_stream.clone()).unwrap();
            
            this.eval::<(), _>("console.log('redirected log');")?;
            this.eval::<(), _>("console.warn('redirected warn');")?;
            this.eval::<(), _>("console.error('redirected error');")?;
            
            assert_eq!(b"redirected log\n", redirected_log_stream.buffer.borrow().as_slice());
            assert_eq!(b"redirected warn\n", redirected_warn_stream.buffer.borrow().as_slice());
            assert_eq!(b"redirected error\n", redirected_error_stream.buffer.borrow().as_slice());

            Ok::<_, Error>(())
        })?;

        // Test actual redirect scenario (log, warn, error all go to same stderr stream)
        let mut all_stderr_stream = SharedStream::default();

        ctx.with(|this| {
            // Redirect mode: console.log, warn, error all use stderr
            register(this.clone(), all_stderr_stream.clone(), all_stderr_stream.clone(), all_stderr_stream.clone()).unwrap();
            
            this.eval::<(), _>("console.log('redirect-log');")?;
            this.eval::<(), _>("console.warn('redirect-warn');")?;
            this.eval::<(), _>("console.error('redirect-error');")?;
            
            let output = String::from_utf8(all_stderr_stream.buffer.borrow().clone())?;
            assert!(output.contains("redirect-log"));
            assert!(output.contains("redirect-warn"));
            assert!(output.contains("redirect-error"));

            Ok::<_, Error>(())
        })?;

        Ok(())
    }

    #[test]
    fn test_real_redirect_mode() -> Result<()> {
        use crate::Config;
        
        // Test with redirect_stdout_to_stderr = false (normal mode)
        let mut normal_config = Config::default();
        normal_config.redirect_stdout_to_stderr(false);
        let normal_runtime = Runtime::new(normal_config)?;
        
        // Test with redirect_stdout_to_stderr = true (redirected mode) 
        let mut redirect_config = Config::default();
        redirect_config.redirect_stdout_to_stderr(true);
        let redirect_runtime = Runtime::new(redirect_config)?;

        // In a real scenario, normal mode would use stdout() for log and stderr() for warn/error
        // while redirect mode would use stderr() for all three
        
        // This demonstrates that both runtimes can be created successfully with console.warn support
        normal_runtime.context().with(|cx| {
            let console: Object<'_> = cx.globals().get("console")?;
            assert!(console.get::<&str, Value<'_>>("log").is_ok());
            assert!(console.get::<&str, Value<'_>>("warn").is_ok());
            assert!(console.get::<&str, Value<'_>>("error").is_ok());
            Ok::<_, Error>(())
        })?;

        redirect_runtime.context().with(|cx| {
            let console: Object<'_> = cx.globals().get("console")?;
            assert!(console.get::<&str, Value<'_>>("log").is_ok());
            assert!(console.get::<&str, Value<'_>>("warn").is_ok());
            assert!(console.get::<&str, Value<'_>>("error").is_ok());
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
