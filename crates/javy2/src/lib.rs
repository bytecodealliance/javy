use std::io::Write;

use anyhow::Result;
use quickjs_wasm_rs::{JSContextRef, JSValueRef};

pub trait JSApiSet {
    fn register<T: Write + 'static, U: Write + 'static>(
        &self,
        context: &JSContextRef,
        config: &'static mut ApiConfig<T, U>,
    ) -> Result<()>;
}

pub struct ApiConfig<T, U> {
    log_stream: T,
    error_stream: U,
}

pub struct Console {}

impl JSApiSet for Console {
    fn register<T, U>(
        &self,
        context: &JSContextRef,
        config: &'static mut ApiConfig<T, U>,
    ) -> Result<()>
    where
        T: Write + 'static,
        U: Write + 'static,
    {
        let global = context.global_object()?;

        let console_log_callback = context.wrap_callback(console_log_to(&mut config.log_stream))?;
        let console_error_callback =
            context.wrap_callback(console_log_to(&mut config.error_stream))?;
        let console_object = context.object_value()?;
        console_object.set_property("log", console_log_callback)?;
        console_object.set_property("error", console_error_callback)?;
        global.set_property("console", console_object)?;
        Ok(())
    }
}

pub struct Runtime<'a> {
    pub context: &'a JSContextRef,
}

pub fn register_apis<T, U>(runtime: &Runtime, config: &'static mut ApiConfig<T, U>) -> Result<()>
where
    T: Write + 'static,
    U: Write + 'static,
{
    let console = Console {};
    console.register(runtime.context, config)?;
    Ok(())
}

fn console_log_to<T>(
    stream: &mut T,
) -> impl FnMut(&JSContextRef, &JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValueRef> + '_
where
    T: Write + 'static,
{
    move |ctx: &JSContextRef, _this: &JSValueRef, args: &[JSValueRef]| {
        // Write full string to in-memory destination before writing to stream since each write call to the stream
        // will invoke a hostcall.
        let mut log_line = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                log_line.push(' ');
            }

            log_line.push_str(arg.as_str()?);
        }

        writeln!(stream, "{log_line}")?;
        ctx.undefined_value()
    }
}
