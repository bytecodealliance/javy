use anyhow::anyhow;
use quickjs_wasm_rs::sys::{
    ext_js_exception, ext_js_undefined, JSContext, JSValue, JS_FreeCString, JS_ToCStringLen2,
    JS_size_t,
};
use quickjs_wasm_rs::{Context, Value};
use std::borrow::Cow;
use std::ffi::c_int;
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

    let console_log_callback = context.new_callback(console_log_to(log_stream))?;
    let console_error_callback = context.new_callback(console_log_to(error_stream))?;
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
        context.wrap_callback(|ctx, _this_arg, args| {
            let (mut fd, data) = js_args_to_io_writer(args)?;
            let n = fd.write(data)?;
            fd.flush()?;
            ctx.value_from_i32(n.try_into()?)
        })?,
    )?;

    global.set_property(
        "__javy_io_readSync",
        context.wrap_callback(|ctx, _this_arg, args| {
            let (mut fd, data) = js_args_to_io_reader(args)?;
            let n = fd.read(data)?;
            ctx.value_from_i32(n.try_into()?)
        })?,
    )?;
    Ok(())
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue + 'static
where
    T: Write + 'static,
{
    move |ctx: *mut JSContext, _this: JSValue, argc: c_int, argv: *mut JSValue, _magic: c_int| {
        let mut len: JS_size_t = 0;
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

fn decode_utf8_buffer_to_js_string(
) -> impl FnMut(&Context, &Value, &[Value]) -> anyhow::Result<Value> {
    move |ctx: &Context, _this: &Value, args: &[Value]| {
        if args.len() != 4 {
            return Err(anyhow!("Expecting 4 arguments, received {}", args.len()));
        }

        let buffer = args[0].as_bytes()?;
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

        let view = buffer
            .get(byte_offset..(byte_offset + byte_length))
            .ok_or(anyhow!(
                "Provided offset and length is not valid for provided buffer"
            ))?;
        let str = if fatal {
            Cow::from(str::from_utf8(view).map_err(|_| anyhow!("The encoded data was not valid"))?)
        } else {
            String::from_utf8_lossy(view)
        };
        ctx.value_from_str(&str)
    }
}

fn encode_js_string_to_utf8_buffer(
) -> impl FnMut(&Context, &Value, &[Value]) -> anyhow::Result<Value> {
    move |ctx: &Context, _this: &Value, args: &[Value]| {
        if args.len() != 1 {
            return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
        }

        let js_string = args[0].as_str()?;
        ctx.array_buffer_value(js_string.as_bytes())
    }
}

fn js_args_to_io_writer(args: &[Value]) -> anyhow::Result<(Box<dyn Write>, &[u8])> {
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

fn js_args_to_io_reader(args: &[Value]) -> anyhow::Result<(Box<dyn Read>, &mut [u8])> {
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
    use std::io;
    use std::rc::Rc;

    #[test]
    fn test_console_log() -> Result<()> {
        let mut stream = SharedStream::default();

        let ctx = Context::default();
        inject_javy_globals(&ctx, stream.clone(), stream.clone())?;

        ctx.eval_global("main", "console.log(\"hello world\");")?;
        assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

        stream.clear();

        ctx.eval_global("main", "console.log(\"bonjour\", \"le\", \"monde\")")?;
        assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());
        Ok(())
    }

    #[test]
    fn test_console_error() -> Result<()> {
        let mut stream = SharedStream::default();

        let ctx = Context::default();
        inject_javy_globals(&ctx, stream.clone(), stream.clone())?;

        ctx.eval_global("main", "console.error(\"hello world\");")?;
        assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

        stream.clear();

        ctx.eval_global("main", "console.error(\"bonjour\", \"le\", \"monde\")")?;
        assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());
        Ok(())
    }

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
}
