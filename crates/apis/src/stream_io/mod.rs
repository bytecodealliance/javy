use anyhow::{anyhow, bail, Error, Result};
use std::io::{Read, Stdin, Write};

use javy::{
    hold, hold_and_release,
    quickjs::{Function, Object, Value},
    to_js_error, Args, Runtime,
};

use crate::{APIConfig, JSApiSet};

pub(super) struct StreamIO;

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
            "Wrong file descriptor: {}. Only stdin(1) and stderr(2) are supported",
            x
        ),
    };
    let data = data
        .as_object()
        .ok_or_else(|| anyhow!("Data must be an Object"))?
        .as_typed_array::<u8>()
        .ok_or_else(|| anyhow!("Data must be a UInt8Array"))?
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
        x => anyhow::bail!("Wrong file descriptor: {}. Only stdin(1) is supported", x),
    };

    let offset = offset
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let length = length
        .as_number()
        .ok_or_else(|| anyhow!("length must be a number"))? as usize;

    let data = data
        .as_object()
        .ok_or_else(|| anyhow!("Data must be an Object"))?
        .as_typed_array::<u8>()
        .ok_or_else(|| anyhow!("Data must be a UInt8Array"))?
        .as_bytes()
        .ok_or_else(|| anyhow!("Could not represent data as &[u8]"))?;

    // Safety
    // This is one of the unfortunate unsafe pieces of the APIs, currently.
    // This should ideally be revisited in order to make it safe.
    // This is unsafe only if the length of the buffer doesn't match the length
    // and offset passed as arguments, the caller must ensure that this is true.
    // We could make this API safe by changing the expectations of the
    // JavaScript side of things in `io.js`.
    let dst = data.as_ptr() as *mut _;
    let dst: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(dst, length) };
    let n = fd.read(&mut dst[offset..(offset + length)])?;

    Ok(Value::new_number(cx, n as f64))
}

impl JSApiSet for StreamIO {
    fn register<'js>(&self, runtime: &Runtime, _config: &APIConfig) -> Result<()> {
        runtime.context().with(|this| {
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

            this.eval(include_str!("io.js"))?;
            Ok::<_, Error>(())
        })?;

        Ok(())
    }
}
