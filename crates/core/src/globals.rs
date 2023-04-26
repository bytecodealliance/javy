use anyhow::anyhow;
use javy::quickjs::{CallbackArg, JSContextRef, JSError, JSValue};
use javy::Runtime;
use std::borrow::Cow;
use std::io::{Read, Write};
use std::str;

pub fn inject_javy_globals<T1, T2>(
    runtime: &Runtime,
    log_stream: T1,
    error_stream: T2,
) -> anyhow::Result<()>
where
    T1: Write + 'static,
    T2: Write + 'static,
{
    let context = runtime.context();
    let global = context.global_object()?;

    let console_log_callback = context.wrap_callback(console_log_to(log_stream))?;
    let console_error_callback = context.wrap_callback(console_log_to(error_stream))?;
    let console_object = context.object_value()?;
    console_object.set_property("log", console_log_callback)?;
    console_object.set_property("error", console_error_callback)?;
    global.set_property("console", console_object)?;

    let javy_object = context.object_value()?;
    global.set_property("Javy", javy_object)?;

    global.set_property(
        "__javy_decodeUtf8BufferToString",
        context.wrap_callback(decode_utf8_buffer_to_js_string())?,
    )?;
    global.set_property(
        "__javy_encodeStringToUtf8Buffer",
        context.wrap_callback(encode_js_string_to_utf8_buffer())?,
    )?;

    global.set_property(
        "__javy_io_writeSync",
        context.wrap_callback(|_, _this_arg, args| {
            let [fd, data, offset, length, ..] = args else {
                anyhow::bail!("Invalid number of parameters");
            };

            let mut fd: Box<dyn Write> = match fd.try_into()? {
                1 => Box::new(std::io::stdout()),
                2 => Box::new(std::io::stderr()),
                _ => anyhow::bail!("Only stdout and stderr are supported"),
            };
            let data: Vec<u8> = data.try_into()?;
            let offset: usize = offset.try_into()?;
            let length: usize = length.try_into()?;
            let data = &data[offset..(offset + length)];
            let n = fd.write(data)?;
            fd.flush()?;
            Ok(n.into())
        })?,
    )?;

    global.set_property(
        "__javy_io_readSync",
        context.wrap_callback(|_, _this_arg, args| {
            let [fd, data, offset, length, ..] = args else {
                anyhow::bail!("Invalid number of parameters");
            };
            let mut fd: Box<dyn Read> = match fd.try_into()? {
                0 => Box::new(std::io::stdin()),
                _ => anyhow::bail!("Only stdin is supported"),
            };
            let offset: usize = offset.try_into()?;
            let length: usize = length.try_into()?;
            let data = unsafe { data.inner_value() };
            if !data.is_array_buffer() {
                anyhow::bail!("Data needs to be an ArrayBuffer");
            }
            let data = data.as_bytes_mut()?;
            let data = &mut data[offset..(offset + length)];
            let n = fd.read(data)?;
            Ok(n.into())
        })?,
    )?;

    context.eval_global(
        "text-encoding.js",
        include_str!("../prelude/text-encoding.js"),
    )?;

    context.eval_global("io.js", include_str!("../prelude/io.js"))?;

    Ok(())
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(&JSContextRef, &CallbackArg, &[CallbackArg]) -> anyhow::Result<JSValue>
where
    T: Write + 'static,
{
    move |_ctx: &JSContextRef, _this: &CallbackArg, args: &[CallbackArg]| {
        // Write full string to in-memory destination before writing to stream since each write call to the stream
        // will invoke a hostcall.
        let mut log_line = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                log_line.push(' ');
            }
            let line = arg.to_string();
            log_line.push_str(&line);
        }

        writeln!(stream, "{log_line}")?;

        Ok(JSValue::Undefined)
    }
}

fn decode_utf8_buffer_to_js_string(
) -> impl FnMut(&JSContextRef, &CallbackArg, &[CallbackArg]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: &CallbackArg, args: &[CallbackArg]| {
        if args.len() != 5 {
            return Err(anyhow!("Expecting 5 arguments, received {}", args.len()));
        }

        let buffer: Vec<u8> = args[0].try_into()?;
        let byte_offset: usize = args[1].try_into()?;
        let byte_length: usize = args[2].try_into()?;
        let fatal: bool = args[3].try_into()?;
        let ignore_bom: bool = args[4].try_into()?;

        let mut view = buffer
            .get(byte_offset..(byte_offset + byte_length))
            .ok_or_else(|| {
                anyhow!("Provided offset and length is not valid for provided buffer")
            })?;

        if !ignore_bom {
            view = match view {
                // [0xEF, 0xBB, 0xBF] is the UTF-8 BOM which we want to strip
                [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
                _ => view,
            };
        }

        let str =
            if fatal {
                Cow::from(str::from_utf8(view).map_err(|_| {
                    JSError::Type("The encoded data was not valid utf-8".to_string())
                })?)
            } else {
                String::from_utf8_lossy(view)
            };
        Ok(str.to_string().into())
    }
}

fn encode_js_string_to_utf8_buffer(
) -> impl FnMut(&JSContextRef, &CallbackArg, &[CallbackArg]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: &CallbackArg, args: &[CallbackArg]| {
        if args.len() != 1 {
            return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
        }

        let js_string: String = args[0].try_into()?;
        Ok(js_string.into_bytes().into())
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime;

    use super::inject_javy_globals;
    use anyhow::Result;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::{cmp, io};

    #[test]
    fn test_console_log() -> Result<()> {
        let mut stream = SharedStream::default();

        let runtime = runtime::new_runtime()?;
        let ctx = runtime.context();
        inject_javy_globals(&runtime, stream.clone(), stream.clone())?;

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

        let runtime = runtime::new_runtime()?;
        let ctx = runtime.context();
        inject_javy_globals(&runtime, stream.clone(), stream.clone())?;

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

    #[test]
    fn test_text_encoder_decoder() -> Result<()> {
        let stream = SharedStream::default();
        let runtime = runtime::new_runtime()?;
        let ctx = runtime.context();
        inject_javy_globals(&runtime, stream.clone(), stream.clone())?;
        ctx.eval_global(
            "main",
            "let encoder = new TextEncoder(); let buffer = encoder.encode('hello'); let decoder = new TextDecoder(); console.log(decoder.decode(buffer));"
        )?;
        assert_eq!(b"hello\n", stream.buffer.borrow().as_slice());

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
