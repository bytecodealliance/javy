use std::io::{Read, Write};

use javy::quickjs::{JSContextRef, JSValueRef};

use crate::JSApiSet;

pub(crate) struct StreamIO {}

impl StreamIO {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl JSApiSet for StreamIO {
    fn register(&self, context: &JSContextRef, _config: &crate::APIConfig) -> anyhow::Result<()> {
        let global = context.global_object()?;
        global.set_property("Javy", context.object_value()?)?;

        let global = context.global_object()?;

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

        context.eval_global("io.js", include_str!("../prelude/io.js"))?;
        Ok(())
    }
}

fn js_args_to_io_writer(args: &[JSValueRef]) -> anyhow::Result<(Box<dyn Write>, &[u8])> {
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

fn js_args_to_io_reader(args: &[JSValueRef]) -> anyhow::Result<(Box<dyn Read>, &mut [u8])> {
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
