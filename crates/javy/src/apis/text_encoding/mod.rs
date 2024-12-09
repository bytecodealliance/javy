use std::str;

use crate::{
    hold, hold_and_release,
    quickjs::{
        context::EvalOptions, Ctx, Exception, Function, String as JSString, TypedArray, Value,
    },
    to_js_error, to_string_lossy, Args,
};
use anyhow::{anyhow, bail, Error, Result};

/// Register `TextDecoder` and `TextEncoder` classes.
pub(crate) fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();
    globals.set(
        "__javy_decodeUtf8BufferToString",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            decode(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;
    globals.set(
        "__javy_encodeStringToUtf8Buffer",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            encode(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;
    let mut opts = EvalOptions::default();
    opts.strict = false;
    this.eval_with_options::<(), _>(include_str!("./text-encoding.js"), opts)?;

    Ok::<_, Error>(())
}

/// Decode a UTF-8 byte buffer as a JavaScript String.
fn decode(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, args) = args.release();
    if args.len() != 5 {
        bail!(
            "Wrong number of arguments. Expected 5 arguments. Got: {}",
            args.len()
        );
    }

    let buffer = args[0]
        .as_object()
        .ok_or_else(|| anyhow!("buffer must be an object"))?
        .as_array_buffer()
        .ok_or_else(|| anyhow!("buffer must be an ArrayBuffer"))?
        .as_bytes()
        .ok_or_else(|| anyhow!("Couldn't retrive &[u8] from buffer"))?;

    let byte_offset = args[1]
        .as_number()
        .ok_or_else(|| anyhow!("offset must be a number"))? as usize;
    let byte_length = args[2]
        .as_number()
        .ok_or_else(|| anyhow!("byte_length must be a number"))? as usize;
    let fatal = args[3]
        .as_bool()
        .ok_or_else(|| anyhow!("fatal must be a boolean"))?;
    let ignore_bom = args[4]
        .as_bool()
        .ok_or_else(|| anyhow!("ignore_bom must be a boolean"))?;

    let mut view = buffer
        .get(byte_offset..(byte_offset + byte_length))
        .ok_or_else(|| anyhow!("Provided offset and length is not valid for provided buffer"))?;

    if !ignore_bom {
        view = match view {
            // [0xEF, 0xBB, 0xBF] is the UTF-8 BOM which we want to strip
            [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
            _ => view,
        };
    }

    let js_string = if fatal {
        JSString::from_str(
            cx.clone(),
            str::from_utf8(view)
                .map_err(|_| Exception::throw_type(&cx, "The encoded data was not valid utf-8"))?,
        )
    } else {
        let str = String::from_utf8_lossy(view);
        JSString::from_str(cx, &str)
    };

    Ok(Value::from_string(js_string?))
}

/// Encode a JavaScript String into a JavaScript UInt8Array.
fn encode(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, args) = args.release();
    if args.len() != 1 {
        bail!("Wrong number of arguments. Expected 1. Got {}", args.len());
    }

    let js_string = args[0]
        .as_string()
        .ok_or_else(|| anyhow!("Argument must be a String"))?;
    let encoded = js_string
        // This is the fast path.
        // The string is already utf-8.
        .to_string()
        .unwrap_or_else(|error| to_string_lossy(&cx, js_string, error));

    Ok(TypedArray::new(cx, encoded.into_bytes())?
        .as_value()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use crate::{quickjs::Value, Config, Runtime};
    use anyhow::{Error, Result};

    #[test]
    fn test_text_encoder_decoder() -> Result<()> {
        let mut config = Config::default();
        config.text_encoding(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result: Value<'_> = this.eval(
                r#"
                let encoder = new TextEncoder(); 
                let decoder = new TextDecoder();

                let buffer = encoder.encode('hello');
                decoder.decode(buffer) == 'hello';
            "#,
            )?;

            assert!(result.as_bool().unwrap());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}
