use std::str;

use anyhow::{anyhow, bail, Error, Result};
use javy::{
    quickjs::{Error as JSError, Function, String as JSString, TypedArray, Value},
    Runtime,
};

use crate::{APIConfig, Args, JSApiSet};

pub(super) struct TextEncoding;

impl JSApiSet for TextEncoding {
    fn register<'js>(&self, runtime: &Runtime, _: &APIConfig) -> Result<()> {
        runtime.context().with(|this| {
            let globals = this.globals();
            globals.set(
                "__javy_decodeUtf8BufferToString",
                Function::new(this.clone(), |cx, args| {
                    decode(Args::hold(cx, args)).expect("decode to succeed")
                }),
            )?;
            globals.set(
                "__javy_encodeStringToUtf8Buffer",
                Function::new(this.clone(), |cx, args| {
                    encode(Args::hold(cx, args)).expect("encode to succeed")
                }),
            )?;
            this.eval(include_str!("./text-encoding.js"))?;
            Ok::<_, Error>(())
        })?;

        Ok(())
    }
}

/// Decode a UTF-8 byte buffer as a JavaScript String.
fn decode<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (cx, args) = args.release();
    if args.len() != 5 {
        bail!(
            "Wrong number of arguments. Expected 5 arguments. Got: {}",
            args.len()
        );
    }

    let buffer = args[0]
        .as_array()
        .ok_or_else(|| anyhow!("buffer must be an array"))?
        .as_typed_array::<u8>()
        .ok_or_else(|| anyhow!("buffer must be a UInt8Array"))?
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
        JSString::from_str(cx, str::from_utf8(view).map_err(JSError::Utf8)?)
    } else {
        let str = String::from_utf8_lossy(view);
        JSString::from_str(cx, &str)
    };

    Ok(Value::from_string(js_string?))
}

/// Encode a JavaScript String into a JavaScript UInt8Array.
fn encode<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (cx, args) = args.release();
    if args.len() != 1 {
        bail!("Wrong number of arguments. Expected 1. Got {}", args.len());
    }

    let js_string = args[0]
        .as_string()
        .ok_or_else(|| anyhow!("Argument must be a String"))?
        .to_string()?;

    Ok(TypedArray::new(cx, js_string.into_bytes())?
        .as_value()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use crate::{APIConfig, JSApiSet};
    use anyhow::{Error, Result};
    use javy::{quickjs::Value, Runtime};

    use super::TextEncoding;

    #[test]
    fn test_text_encoder_decoder() -> Result<()> {
        let runtime = Runtime::default();

        runtime.context().with(|this| {

            TextEncoding.register(&runtime, &APIConfig::default())?;
            let result: Value<'_> = this.eval(
                "let encoder = new TextEncoder(); let buffer = encoder.encode('hello'); let decoder = new TextDecoder(); decoder.decode(buffer) == 'hello';"
            )?;
            assert!(result.as_bool().unwrap());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}
