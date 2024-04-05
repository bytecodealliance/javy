use anyhow::{anyhow, bail, Error, Result};
use std::io::{Read, Write};

use javy::{
    quickjs::{Function, Object, Value},
    Runtime,
};

use crate::{APIConfig, Args, JSApiSet};

pub(super) struct StreamIO;

fn ensure_io_args<'a, 'js: 'a>(
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

fn write<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (cx, args) = args.release();
    let (fd, data, offset, length) = ensure_io_args(&args, "Javy.IO.writeSync")?;
    let mut fd: Box<dyn Write> = match fd
        .as_int()
        .ok_or_else(|| anyhow!("File descriptor must be a number"))?
    {
        // TODO: Drop the `Box` to avoid a heap allocation?
        1 => Box::new(std::io::stdout()),
        2 => Box::new(std::io::stderr()),
        x => anyhow::bail!(
            "Wrong file descriptor: {}. Only stdin(1) and stderr(2) are supported",
            x
        ),
    };
    let data = data
        .as_array()
        .ok_or_else(|| anyhow!("Data must be an Array object"))?
        .as_typed_array::<u8>()
        .ok_or_else(|| anyhow!("Data must be a UInt8Array"))?
        .as_bytes()
        .ok_or_else(|| anyhow!("Could not represent data as &[u8]"))?;

    // TODO: Revisit the f64 to usize conversions.
    let offset = offset
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let length = length
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let data = &data[offset..(offset + length)];
    let n = fd.write(data)?;
    fd.flush()?;

    Ok(Value::new_number(cx, n as f64))
}

fn read<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (cx, args) = args.release();
    let (fd, data, offset, length) = ensure_io_args(&args, "Javy.IO.readSync")?;

    let mut fd: Box<dyn Read> = match fd
        .as_int()
        .ok_or_else(|| anyhow!("File descriptor must be a number"))?
    {
        // TODO: Drop the `Box` to avoid a heap allocation?
        0 => Box::new(std::io::stdin()),
        x => anyhow::bail!("Wrong file descriptor: {}. Only stdin(1) is supported", x),
    };

    let offset = offset
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let length = length
        .as_number()
        .ok_or_else(|| anyhow!("length must be a number"))? as usize;

    let data = data
        .as_array()
        .ok_or_else(|| anyhow!("Data must be an Array object"))?
        .as_typed_array::<u8>()
        .ok_or_else(|| anyhow!("Data must be a UInt8Array"))?
        .as_bytes()
        .ok_or_else(|| anyhow!("Could not represent data as &[u8]"))?;

    let len = data.len();
    let mut_ptr = data.as_ptr() as *mut u8;
    // TODO: Can we avoid doing this? Is there a way to expose a safe way to mutate
    // the underlying array buffer with rquickjs?
    let mut_data = unsafe { &mut *std::ptr::slice_from_raw_parts_mut(mut_ptr, len) };

    let data = &mut mut_data[offset..(offset + length)];
    let n = fd.read(data)?;

    Ok(Value::new_number(cx, n as f64))
}

impl JSApiSet for StreamIO {
    fn register<'js>(&self, runtime: &Runtime, _config: &APIConfig) -> Result<()> {
        runtime.context().with(|this| {
            let globals = this.globals();
            // TODO: Do we need this?
            if globals.get::<_, Object>("Javy").is_err() {
                globals.set("Javy", Object::new(this.clone())?)?
            }

            globals.set(
                "__javy_io_writeSync",
                Function::new(this.clone(), |cx, args| {
                    write(Args::hold(cx, args)).expect("write to succeed")
                }),
            )?;

            globals.set(
                "__javy_io_readSync",
                Function::new(this.clone(), |cx, args| {
                    read(Args::hold(cx, args)).expect("read to succeed")
                }),
            )?;

            this.eval(include_str!("io.js"))?;
            Ok::<_, Error>(())
        })?;

        Ok(())
    }
}
