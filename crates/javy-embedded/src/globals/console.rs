use std::io::{self, Write};

use anyhow::Result;
use quickjs_wasm_rs::{Context, Value};


pub fn add(context: &Context) -> Result<()> {
    let global = context.global_object()?;
    let console_object = context.object_value()?;
    console_object.set_property("log", context.wrap_callback(console_log_to(io::stdout()))?)?;
    console_object.set_property("error", context.wrap_callback(console_log_to(io::stderr()))?)?;
    global.set_property("console", console_object)?;
    Ok(())
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(&Context, &Value, &[Value]) -> anyhow::Result<Value>
where
    T: Write + 'static,
{
    move |ctx: &Context, _this: &Value, args: &[Value]| {
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