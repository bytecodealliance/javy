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
        function::This,
        prelude::{MutFn, Rest},
        Ctx, Exception, Function, Object, String as JSString, Value,
    },
    to_js_error, val_to_string, Args,
};

use crate::serde::de::get_to_json;

use simd_json::Error as SError;

use anyhow::{anyhow, bail, Result};
use std::{
    io::{Read, Write},
    sync::OnceLock,
    time::SystemTime,
};

static DEFAULT_PARSE_KEY: OnceLock<String> = OnceLock::new();

/// Use SIMD implementations for `JSON.parse` and `JSON.stringify`.
pub(crate) fn register<'js>(this: Ctx<'js>) -> Result<()> {
    let global = this.globals();

    let json: Object = global.get("JSON")?;
    let default_parse: Function = json.get("parse")?;
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .subsec_millis();
    // Make the global key unstable so users can't rely on it being stable.
    let default_parse_key = DEFAULT_PARSE_KEY.get_or_init(|| format!("__javy_{millis}_json_parse"));
    global.set(default_parse_key, default_parse)?;

    let parse = Function::new(
        this.clone(),
        MutFn::new(move |cx: Ctx<'js>, args: Rest<Value<'js>>| {
            call_json_parse(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    // Explicitly set the function's name and length properties.
    // In both the parse and the stringify case below, the spec tests
    // assert that the name and length properties must be  correctly set.
    parse.set_length(2)?;
    parse.set_name("parse")?;

    let stringify = Function::new(
        this.clone(),
        MutFn::new(|cx: Ctx<'js>, args: Rest<Value<'js>>| {
            call_json_stringify(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    stringify.set_name("stringify")?;
    stringify.set_length(3)?;

    let global = this.globals();
    let json: Object = global.get("JSON")?;
    json.set("parse", parse)?;
    json.set("stringify", stringify)?;

    Ok(())
}

fn call_json_parse(args: Args<'_>) -> Result<Value<'_>> {
    let (this, args) = args.release();

    match args.len() {
        0 => bail!(Exception::throw_syntax(
            &this,
            "\"undefined\" is not valid JSON"
        )),
        1 => {
            let val = args[0].clone();
            // Fast path. Number and null are treated as identity.
            if val.is_number() || val.is_null() {
                return Ok(val);
            }

            if val.is_symbol() {
                bail!(Exception::throw_type(&this, "Expected string primitive"));
            }

            let mut string = val_to_string(&this, args[0].clone())?;
            let bytes = unsafe { string.as_bytes_mut() };
            json::parse(this.clone(), bytes).map_err(|original| {
                if original.downcast_ref::<SError>().is_none() {
                    return original;
                }

                let e = match original.downcast_ref::<SError>() {
                    Some(e) => e.to_string(),
                    None => "JSON parse error".into(),
                };
                anyhow!(Exception::throw_syntax(&this, &e))
            })
        }
        _ => {
            // If there's more than one argument, defer to the built-in
            // JSON.parse, which will take care of validating and invoking the
            // reviver argument.
            //
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse#reviver.
            let default: Function = this.globals().get(DEFAULT_PARSE_KEY.get().unwrap())?;
            default
                .call((args[0].clone(), args[1].clone()))
                .map_err(|e| anyhow!(e))
        }
    }
}

fn call_json_stringify(args: Args<'_>) -> Result<Value<'_>> {
    let (this, args) = args.release();

    match args.len() {
        0 => Ok(Value::new_undefined(this.clone())),
        1 => {
            let arg = args[0].clone();
            let val: Value = if arg.is_object() {
                if let Some(f) = get_to_json(&arg) {
                    f.call((
                        This(arg.clone()),
                        JSString::from_str(arg.ctx().clone(), "")?.into_value(),
                    ))?
                } else {
                    arg.clone()
                }
            } else {
                arg.clone()
            };
            if val.is_function() || val.is_undefined() || val.is_symbol() {
                return Ok(Value::new_undefined(arg.ctx().clone()));
            }

            let bytes = json::stringify(val.clone())?;
            let str = String::from_utf8(bytes)?;
            let str = JSString::from_str(this, &str)?;
            Ok(str.into_value())
        }
        2 => Ok(this
            .json_stringify_replacer(args[0].clone(), args[1].clone())?
            .map_or_else(
                || Value::new_undefined(this.clone()),
                |str| str.into_value(),
            )),
        _ => Ok(this
            .json_stringify_replacer_space(args[0].clone(), args[1].clone(), args[2].clone())?
            .map_or_else(
                || Value::new_undefined(this.clone()),
                |str| str.into_value(),
            )),
    }
}

/// Register `Javy.JSON.fromStdin` and `Javy.JSON.toStdout` functions on the
/// global object.
pub(crate) fn register_javy_json(this: Ctx<'_>) -> Result<()> {
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
fn from_stdin(args: Args<'_>) -> Result<Value> {
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
fn to_stdout(args: Args<'_>) -> Result<()> {
    let (_, args) = args.release();
    let mut fd = std::io::stdout();
    let buffer = json::stringify(args[0].clone())?;
    fd.write_all(&buffer)?;
    fd.flush().map_err(Into::into)
}
