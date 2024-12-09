use anyhow::{anyhow, bail, Error, Result};
use std::io::{Read, Stdin, Write};

use crate::{
    hold, hold_and_release,
    quickjs::{qjs::JS_GetArrayBuffer, Ctx, Function, Object, Value},
    to_js_error, Args,
};

/// Register `Javy.IO.readSync` and `Javy.IO.writeSync` functions on the
/// global object.
pub(crate) fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();
    if globals.get::<_, Object>("Javy").is_err() {
        globals.set("Javy", Object::new(this.clone())?)?
    }

    globals.set(
        "__javy_io_writeSync",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            write(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    globals.set(
        "__javy_io_readSync",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            read(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    this.eval::<(), _>(include_str!("io.js"))?;
    Ok::<_, Error>(())
}

fn extract_args<'a, 'js: 'a>(
    args: &'a [Value<'js>],
    for_func: &str,
) -> Result<(
    &'a Value<'js>,
    &'a Value<'js>,
    &'a Value<'js>,
    &'a Value<'js>,
)> {
    let [fd, data, offset, length, ..] = args else {
        bail!(
            r#"
           {} expects 4 parameters: the file descriptor, the
           TypedArray buffer, the TypedArray byteOffset and the TypedArray
           byteLength.

           Got: {} parameters.
        "#,
            for_func,
            args.len()
        );
    };

    Ok((fd, data, offset, length))
}

fn write(args: Args<'_>) -> Result<Value<'_>> {
    enum Fd {
        Stdout,
        Stderr,
    }

    let (cx, args) = args.release();
    let (fd, data, offset, length) = extract_args(&args, "Javy.IO.writeSync")?;
    let fd = match fd
        .as_int()
        .ok_or_else(|| anyhow!("File descriptor must be a number"))?
    {
        1 => Fd::Stdout,
        2 => Fd::Stderr,
        x => anyhow::bail!(
            "Unsupported file descriptor: {x}. Only stdout(1) and stderr(2) are supported"
        ),
    };
    let data = data
        .as_object()
        .ok_or_else(|| anyhow!("Data must be an Object"))?
        .as_array_buffer()
        .ok_or_else(|| anyhow!("Data must be an ArrayBuffer"))?
        .as_bytes()
        .ok_or_else(|| anyhow!("Could not represent data as &[u8]"))?;

    let offset = offset
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let length = length
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let data = &data[offset..(offset + length)];
    let n = match fd {
        Fd::Stdout => {
            let mut fd = std::io::stdout();
            let n = fd.write(data)?;
            fd.flush()?;
            n
        }
        Fd::Stderr => {
            let mut fd = std::io::stderr();
            let n = fd.write(data)?;
            fd.flush()?;
            n
        }
    };

    Ok(Value::new_number(cx, n as f64))
}

fn read(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, args) = args.release();
    let (fd, data, offset, length) = extract_args(&args, "Javy.IO.readSync")?;

    let mut fd: Stdin = match fd
        .as_int()
        .ok_or_else(|| anyhow!("File descriptor must be a number"))?
    {
        0 => std::io::stdin(),
        x => anyhow::bail!("Unsupported file descriptor: {x}. Only stdin(0) is supported"),
    };

    let offset = offset
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let length = length
        .as_number()
        .ok_or_else(|| anyhow!("length must be a number"))? as usize;

    // Safety
    // This is one of the unfortunate unsafe pieces of the APIs, currently.
    // This is a port of the previous implementation.
    // This should ideally be revisited in order to make it safe.
    // This is unsafe only if the length of the buffer doesn't match the length
    // and offset passed as arguments, the caller must ensure that this is true.
    // We could make this API safe by changing the expectations of the
    // JavaScript side of things in `io.js`.
    let data = unsafe {
        let mut len = 0;
        let ptr = JS_GetArrayBuffer(cx.as_raw().as_ptr(), &mut len, data.as_raw());
        if ptr.is_null() {
            bail!("Data must be an ArrayBuffer");
        }

        Ok::<_, Error>(std::slice::from_raw_parts_mut(ptr, len as _))
    }?;

    let data = &mut data[offset..(offset + length)];
    let n = fd.read(data)?;

    Ok(Value::new_number(cx, n as f64))
}
