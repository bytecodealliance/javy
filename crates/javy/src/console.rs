use anyhow::Result;
use std::io::{self, Write};

use quickjs_wasm_rs::{JSContextRef, JSValueRef};

use crate::JSApiSet;

pub enum LogStream {
    StdOut,
    StdErr,
}

impl LogStream {
    fn into_stream(&self) -> Box<dyn Write + 'static> {
        match self {
            Self::StdErr => Box::new(io::stderr()),
            Self::StdOut => Box::new(io::stdout()),
        }
    }
}

pub(crate) struct Console {}

impl Console {
    pub(crate) fn new() -> Self {
        Console {}
    }
}

impl JSApiSet for Console {
    fn register(&self, context: &JSContextRef, config: &crate::Config) -> Result<()> {
        let global = context.global_object()?;

        let console_log_callback =
            context.wrap_callback(console_log_to(config.log_stream.into_stream()))?;
        let console_error_callback =
            context.wrap_callback(console_log_to(config.error_stream.into_stream()))?;
        let console_object = context.object_value()?;
        console_object.set_property("log", console_log_callback)?;
        console_object.set_property("error", console_error_callback)?;
        global.set_property("console", console_object)?;
        Ok(())
    }
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(&JSContextRef, &JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValueRef>
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
