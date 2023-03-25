use anyhow::anyhow;
use quickjs_wasm_rs::{Context, JSError, Value, JavyValue};
use std::borrow::Cow;
use std::io::{Read, Write};
use std::str;

pub fn inject_javy_globals<T1, T2>(
    context: &Context,
    log_stream: T1,
    error_stream: T2,
) -> anyhow::Result<()>
where
    T1: Write + 'static,
    T2: Write + 'static,
{
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
    // TODO
    // global.set_property(
    //     "__javy_encodeStringToUtf8Buffer",
    //     context.wrap_callback(encode_js_string_to_utf8_buffer())?,
    // )?;

    global.set_property(
        "__javy_io_writeSync",
        context.wrap_callback(|_, _this_arg, args| {
            let (mut fd, data) = js_args_to_io_writer(args)?;
            let n = fd.write(data)?;
            fd.flush()?;
            Ok(JavyValue::Int(n as i32))
        })?,
    )?;

    global.set_property(
        "__javy_io_readSync",
        context.wrap_callback(|_, _this_arg, args| {
            let (mut fd, data) = js_args_to_io_reader(args)?;
            let n = fd.read(data)?;
            Ok(JavyValue::Int(n as i32))
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
) -> impl FnMut(&Context, &JavyValue, &[JavyValue]) -> anyhow::Result<JavyValue>
where
    T: Write + 'static,
{
    move |_: &Context, _this: &JavyValue, args: &[JavyValue]| {
        // Write full string to in-memory destination before writing to stream since each write call to the stream
        // will invoke a hostcall.
        let mut log_line = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                log_line.push(' ');
            }
            let str_arg = arg.as_str().unwrap(); // check for None
            log_line.push_str(str_arg);
        }

        writeln!(stream, "{log_line}")?;
        Ok(JavyValue::Undefined)
    }
}

fn decode_utf8_buffer_to_js_string(
) -> impl FnMut(&Context, &JavyValue, &[JavyValue]) -> anyhow::Result<JavyValue> {
    move |_: &Context, _this: &JavyValue, args: &[JavyValue]| {
        if args.len() != 5 {
            return Err(anyhow!("Expecting 5 arguments, received {}", args.len()));
        }

        let buffer = args[0];
        let byte_offset = {
            let byte_offset_val = &args[1];
            if !byte_offset_val.is_repr_as_i32() {
                return Err(anyhow!("byte_offset must be an u32"));
            }
            byte_offset_val.as_u32_unchecked()
        }
        .try_into()?;
        let byte_length: usize = {
            let byte_length_val = &args[2];
            if !byte_length_val.is_repr_as_i32() {
                return Err(anyhow!("byte_length must be an u32"));
            }
            byte_length_val.as_u32_unchecked()
        }
        .try_into()?;
        let fatal = args[3].as_bool()?;
        let ignore_bom = args[4].as_bool()?;

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
        Ok(JavyValue::String(str.to_string()))
    }
}

// fn encode_js_string_to_utf8_buffer(
// ) -> impl FnMut(&Context, &Value, &[Value]) -> anyhow::Result<JavyValue> {
//     move |ctx: &Context, _this: &Value, args: &[Value]| {
//         if args.len() != 1 {
//             return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
//         }

//         let js_string = args[0].as_str_lossy();
//         ctx.array_buffer_value(js_string.as_bytes()) // TODO: Need to implement serde for Arrays
//     }
// }

fn js_args_to_io_writer<'a> (args: &'a [Value<'a>]) -> anyhow::Result<(Box<dyn Write>, &'a [u8])> {
    // TODO: Should throw an exception
    let [fd, data, offset, length, ..] = args else {
        anyhow::bail!("Invalid number of parameters");
    };

    let offset: usize = (offset.as_f64()?.floor() as u64).try_into()?;
    let length: usize = (length.as_f64()?.floor() as u64).try_into()?;

    let fd: Box<dyn Write> = match fd.try_as_integer()? {
        1 => Box::new(std::io::stdout()),
        2 => Box::new(std::io::stderr()),
        _ => anyhow::bail!("Only stdout and stderr are supported"),
    };

    if !data.is_array_buffer() {
        anyhow::bail!("Data needs to be an ArrayBuffer");
    }
    let data = data.as_bytes()?;
    Ok((fd, &data[offset..(offset + length)]))
}

fn js_args_to_io_reader<'a>(args: &'a [Value<'a>]) -> anyhow::Result<(Box<dyn Read>, &'a mut [u8])> {
    // TODO: Should throw an exception
    let [fd, data, offset, length, ..] = args else {
        anyhow::bail!("Invalid number of parameters");
    };

    let offset: usize = (offset.as_f64()?.floor() as u64).try_into()?;
    let length: usize = (length.as_f64()?.floor() as u64).try_into()?;

    let fd: Box<dyn Read> = match fd.try_as_integer()? {
        0 => Box::new(std::io::stdin()),
        _ => anyhow::bail!("Only stdin is supported"),
    };

    if !data.is_array_buffer() {
        anyhow::bail!("Data needs to be an ArrayBuffer");
    }
    let data = data.as_bytes_mut()?;
    Ok((fd, &mut data[offset..(offset + length)]))
}

#[cfg(test)]
mod tests {
    use super::inject_javy_globals;
    use anyhow::Result;
    use quickjs_wasm_rs::Context;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::{cmp, io};

    #[test]
    fn test_console_log() -> Result<()> {
        let mut stream = SharedStream::default();

        let ctx = Context::default();
        inject_javy_globals(&ctx, stream.clone(), stream.clone())?;

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

        let ctx = Context::default();
        inject_javy_globals(&ctx, stream.clone(), stream.clone())?;

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
