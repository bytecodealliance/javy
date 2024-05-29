//! High-performance JSON implementation for  Javy.
//!
//! The provided implementation is based on
//! [simd-json](https://crates.io/crates/simd-json)
//!
//! The most efficient combination according to our experiments, is to use:
//! * SIMD JSON for `JSON.parse`
//! * Serde JSON for `JSON.stringify`
//!
//! It's also important to note that this implementation optimizes for the hot
//! path:
//! - If `JSON.parse` is invoked with the reviver argument, the native QuickJS
//!   `JSON.parse` is invoked instead.
//! - If `JSON.stringify` is invoked with the replacer and/or space arguments, the
//!   native QuickJS `JSON.stringify` is invoked instead.
//!
//! The reason behind this decision is simple: most use-cases will hit the
//! hotpath and doing any sort of inline processing of the parsed or stringified
//! values is likely to void any performance benefits.
use crate::{
    hold, hold_and_release, json,
    quickjs::{
        context::Intrinsic,
        prelude::{MutFn, Rest},
        qjs, Ctx, Function, Object, String as JSString, Value,
    },
    to_js_error, val_to_string, Args,
};

use anyhow::{anyhow, Result};
use std::{
    io::{Read, Write},
    ptr::NonNull,
};

/// Intrinsic to attach faster JSON.{parse/stringify} functions.
pub struct Json;

/// Intrinsic to attach functions under the `Javy.JSON` namespace.
pub struct JavyJson;

impl Intrinsic for JavyJson {
    unsafe fn add_intrinsic(ctx: NonNull<qjs::JSContext>) {
        register_javy_json(Ctx::from_raw(ctx)).expect("registering Javy.JSON builtins to succeed")
    }
}

impl Intrinsic for Json {
    unsafe fn add_intrinsic(ctx: NonNull<qjs::JSContext>) {
        register(Ctx::from_raw(ctx)).expect("registering JSON builtins to succeed")
    }
}

fn register<'js>(this: Ctx<'js>) -> Result<()> {
    let parse = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            call_json_parse(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    let stringify = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            call_json_stringify(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    let global = this.globals();
    let json: Object = global.get("JSON")?;
    json.set("parse", parse)?;
    json.set("stringify", stringify)?;

    Ok(())
}

fn call_json_parse<'js>(args: Args<'js>) -> Result<Value> {
    let (this, args) = args.release();

    match args.len() {
        0 => Err(anyhow!("Error: \"undefined\" is not valid JSON")),
        1 => {
            let val = args[0].clone();
            // Fast path. Number and null are treated as identity.
            if val.is_number() || val.is_null() {
                return Ok(val);
            }

            let mut string = val_to_string(this.clone(), args[0].clone())?;
            let mut bytes = unsafe { string.as_bytes_mut() };
            json::parse(this, &mut bytes)
        }
        _ => {
            // If there's more than one argument, defer to the built-in
            // JSON.parse, which will take care of validating and invoking the
            // reviver argument.
            //
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse#reviver.
            let global = this.globals();
            let json: Object = global.get("JSON")?;
            let parse: Function = json.get("parse")?;

            parse
                .call((args[0].clone(), args[1].clone()))
                .map_err(|e| anyhow!("{e}"))
        }
    }
}

fn call_json_stringify<'js>(args: Args<'js>) -> Result<Value> {
    let (this, args) = args.release();

    match args.len() {
        0 => Ok(Value::new_undefined(this.clone())),
        1 => {
            let bytes = json::stringify(args[0].clone())?;
            let str = String::from_utf8(bytes)?;
            let str = JSString::from_str(this, &str)?;
            Ok(str.into_value())
        }
        _ => {
            // Similar to the parse case, if there's more than one argument,
            // defer to the built-in JSON.stringify, which will take care of
            // validating invoking the replacer function and/or applying the
            // space argument.
            let global = this.globals();
            let json: Object = global.get("JSON")?;
            let stringify: Function = json.get("stringify")?;

            if args.len() == 2 {
                stringify
                    .call((args[0].clone(), args[1].clone()))
                    .map_err(|e| anyhow!("{e}"))
            } else {
                stringify
                    .call((args[0].clone(), args[1].clone(), args[2].clone()))
                    .map_err(|e| anyhow!("{e}"))
            }
        }
    }
}

fn register_javy_json<'js>(this: Ctx<'js>) -> Result<()> {
    let globals = this.globals();
    let javy = if globals.get::<_, Object>("Javy").is_err() {
        Object::new(this.clone())?
    } else {
        globals.get::<_, Object>("Javy").unwrap()
    };

    let from_stdin = Function::new(this.clone(), |cx, args| {
        let (cx, args) = hold_and_release!(cx, args);
        from_stdin(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
    });

    let to_stdout = Function::new(this.clone(), |cx, args| {
        let (cx, args) = hold_and_release!(cx, args);
        to_stdout(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
    });

    let json = Object::new(this)?;
    json.set("fromStdin", from_stdin)?;
    json.set("toStdout", to_stdout)?;

    javy.set("JSON", json)?;
    globals.set("Javy", javy).map_err(Into::into)
}

/// Definition for Javy.JSON.fromStdin
fn from_stdin<'js>(args: Args<'js>) -> Result<Value<'js>> {
    // Light experimentation shows that 1k bytes is enough to avoid paying the
    // high relocation costs. We can modify as we see fit or even make this
    // configurable if needed.
    let mut buffer = Vec::with_capacity(1000);
    let mut fd = std::io::stdin();
    fd.read_to_end(&mut buffer)?;
    let (ctx, _) = args.release();
    json::parse(ctx, &mut buffer)
}

/// Definition for Javy.JSON.toStdout
fn to_stdout<'js>(args: Args<'js>) -> Result<()> {
    let (_, args) = args.release();
    let mut fd = std::io::stdout();
    let buffer = json::stringify(args[0].clone())?;
    fd.write_all(&buffer).map_err(Into::into)
}
