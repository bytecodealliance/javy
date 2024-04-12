use std::str;

use crate::{APIConfig, JSApiSet};
use anyhow::{anyhow, bail, Error, Result};
use javy::{
    hold, hold_and_release,
    quickjs::{
        context::EvalOptions, qjs, Error as JSError, Exception, Function, String as JSString,
        TypedArray, Value,
    },
    to_js_error, Args, Runtime,
};

pub(super) struct TextEncoding;

impl JSApiSet for TextEncoding {
    fn register<'js>(&self, runtime: &Runtime, _: &APIConfig) -> Result<()> {
        runtime.context().with(|this| {
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
            let opts = EvalOptions {
                strict: false,
                ..Default::default()
            };
            this.eval_with_options(include_str!("./text-encoding.js"), opts)?;

            Ok::<_, Error>(())
        })?;

        Ok(())
    }
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
        .ok_or_else(|| anyhow!("Argument must be a String"))?
        // This is the fast path.
        // The string is already utf-8.
        .to_string()
        .unwrap_or_else(|error| {
            let mut len: qjs::size_t = 0;
            let ptr = unsafe {
                qjs::JS_ToCStringLen2(cx.as_raw().as_ptr(), &mut len, args[0].as_raw(), 0)
            };
            let buf = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
            to_string_lossy(error, buf)
        });

    Ok(TypedArray::new(cx, js_string.into_bytes())?
        .as_value()
        .to_owned())
}

/// Converts the JavaScript value to a string, replacing any invalid UTF-8 sequences with the
/// Unicode replacement character (U+FFFD).
fn to_string_lossy(error: JSError, buffer: &[u8]) -> String {
    // The error here *must* be a Utf8 error; the `JSString::to_string()` may
    // return `JSError::Unknown`, but at that point, something else has gone
    // wrong too.
    let mut utf8_error = match error {
        JSError::Utf8(e) => e,
        _ => unreachable!("expected Utf8 error"),
    };
    let mut res = String::new();
    let mut buffer = buffer;
    loop {
        let (valid, after_valid) = buffer.split_at(utf8_error.valid_up_to());
        res.push_str(unsafe { str::from_utf8_unchecked(valid) });
        res.push(char::REPLACEMENT_CHARACTER);

        // see https://simonsapin.github.io/wtf-8/#surrogate-byte-sequence
        let lone_surrogate = matches!(after_valid, [0xED, 0xA0..=0xBF, 0x80..=0xBF, ..]);

        // https://simonsapin.github.io/wtf-8/#converting-wtf-8-utf-8 states that each
        // 3-byte lone surrogate sequence should be replaced by 1 UTF-8 replacement
        // char. Rust's `Utf8Error` reports each byte in the lone surrogate byte
        // sequence as a separate error with an `error_len` of 1. Since we insert a
        // replacement char for each error, this results in 3 replacement chars being
        // inserted. So we use an `error_len` of 3 instead of 1 to treat the entire
        // 3-byte sequence as 1 error instead of as 3 errors and only emit 1
        // replacement char.
        let error_len = if lone_surrogate {
            3
        } else {
            utf8_error
                .error_len()
                .expect("Error length should always be available on underlying buffer")
        };

        buffer = &after_valid[error_len..];
        match str::from_utf8(buffer) {
            Ok(valid) => {
                res.push_str(valid);
                break;
            }
            Err(e) => utf8_error = e,
        }
    }
    res
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
        TextEncoding.register(&runtime, &APIConfig::default())?;

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
