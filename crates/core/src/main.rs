use quickjs_wasm_rs::{Context, Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read, Write};

static mut CONTEXT: OnceCell<Context> = OnceCell::new();
static mut CODE: OnceCell<String> = OnceCell::new();

// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let mut context = Context::default();
    context
        .register_globals(io::stderr(), io::stderr())
        .unwrap();

    let global = context.global_object().unwrap();
    inject_javy_globals(&context, &global);

    context
        .eval_global(
            "text-encoding.js",
            include_str!("../prelude/text-encoding.js"),
        )
        .unwrap();

    context
        .eval_global("io.js", include_str!("../prelude/io.js"))
        .unwrap();

    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();

    unsafe {
        CONTEXT.set(context).unwrap();
        CODE.set(contents).unwrap();
    }
}

fn js_args_to_io_writer(args: &[Value]) -> anyhow::Result<(Box<dyn Write>, &[u8])> {
    // TODO: Should throw an exception
    let [fd, data, ..] = args else {
        anyhow::bail!("Invalid number of parameters");
    };

    let fd: Box<dyn Write> = match fd.as_f64()?.floor() as usize {
        1 => Box::new(std::io::stdout()),
        2 => Box::new(std::io::stderr()),
        _ => anyhow::bail!("Only stdout and stderr are supported"),
    };

    if !data.is_array_buffer() {
        anyhow::bail!("Data needs to be an ArrayBuffer");
    }
    let data = data.as_bytes()?;
    Ok((fd, data))
}

fn js_args_to_io_reader(args: &[Value]) -> anyhow::Result<(Box<dyn Read>, &mut [u8])> {
    // TODO: Should throw an exception
    let [fd, data, ..] = args else {
        anyhow::bail!("Invalid number of parameters");
    };

    let fd: Box<dyn Read> = match fd.as_f64()?.floor() as usize {
        0 => Box::new(std::io::stdin()),
        _ => anyhow::bail!("Only stdin is supported"),
    };

    if !data.is_array_buffer() {
        anyhow::bail!("Data needs to be an ArrayBuffer");
    }
    let data = data.as_bytes_mut()?;
    Ok((fd, data))
}

fn inject_javy_globals(context: &Context, global: &Value) {
    global
        .set_property(
            "__javy_io_writeSync",
            context
                .wrap_callback(|ctx, _this_arg, args| {
                    let (mut fd, data) = js_args_to_io_writer(args)?;
                    let n = fd.write(data)?;
                    Ok(ctx.value_from_i32(n.try_into()?)?)
                })
                .unwrap(),
        )
        .unwrap();

    global
        .set_property(
            "__javy_io_readSync",
            context
                .wrap_callback(|ctx, _this_arg, args| {
                    let (mut fd, data) = js_args_to_io_reader(args)?;
                    let n = fd.read(data)?;
                    Ok(ctx.value_from_i32(n.try_into()?)?)
                })
                .unwrap(),
        )
        .unwrap();
}

fn main() {
    let code = unsafe { CODE.take().unwrap() };
    let context = unsafe { CONTEXT.take().unwrap() };

    context.eval_global("function.mjs", &code).unwrap();
}
