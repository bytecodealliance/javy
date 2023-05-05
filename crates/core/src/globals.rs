use anyhow::anyhow;
use javy::quickjs::{CallbackArg, JSContextRef, JSError, JSValue};
use javy::Runtime;
use std::borrow::Cow;
use std::str;

pub fn inject_javy_globals(runtime: &Runtime) -> anyhow::Result<()> {
    let context = runtime.context();
    let global = context.global_object()?;

    global.set_property(
        "__javy_decodeUtf8BufferToString",
        context.wrap_callback(decode_utf8_buffer_to_js_string())?,
    )?;
    global.set_property(
        "__javy_encodeStringToUtf8Buffer",
        context.wrap_callback(encode_js_string_to_utf8_buffer())?,
    )?;

    context.eval_global(
        "text-encoding.js",
        include_str!("../prelude/text-encoding.js"),
    )?;

    Ok(())
}

fn decode_utf8_buffer_to_js_string(
) -> impl FnMut(&JSContextRef, &CallbackArg, &[CallbackArg]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: &CallbackArg, args: &[CallbackArg]| {
        if args.len() != 5 {
            return Err(anyhow!("Expecting 5 arguments, received {}", args.len()));
        }

        let buffer: Vec<u8> = args[0].try_into()?;
        let byte_offset: usize = args[1].try_into()?;
        let byte_length: usize = args[2].try_into()?;
        let fatal: bool = args[3].try_into()?;
        let ignore_bom: bool = args[4].try_into()?;

        let mut view = buffer
            .get(byte_offset..(byte_offset + byte_length))
            .ok_or_else(|| {
                anyhow!("Provided offset and length is not valid for provided buffer")
            })?;

        if !ignore_bom {
            view = match view {
                // [0xEF, 0xBB, 0xBF] is the UTF-8 BOM which we want to strip
                [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
                _ => view,
            };
        }

        let str =
            if fatal {
                Cow::from(str::from_utf8(view).map_err(|_| {
                    JSError::Type("The encoded data was not valid utf-8".to_string())
                })?)
            } else {
                String::from_utf8_lossy(view)
            };
        Ok(str.to_string().into())
    }
}

fn encode_js_string_to_utf8_buffer(
) -> impl FnMut(&JSContextRef, &CallbackArg, &[CallbackArg]) -> anyhow::Result<JSValue> {
    move |_ctx: &JSContextRef, _this: &CallbackArg, args: &[CallbackArg]| {
        if args.len() != 1 {
            return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
        }

        let js_string: String = args[0].try_into()?;
        Ok(js_string.into_bytes().into())
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime;

    use super::inject_javy_globals;
    use anyhow::Result;

    #[test]
    fn test_text_encoder_decoder() -> Result<()> {
        let runtime = runtime::new_runtime()?;
        let ctx = runtime.context();
        inject_javy_globals(&runtime)?;
        ctx.eval_global(
            "main",
            "let encoder = new TextEncoder(); let buffer = encoder.encode('hello'); let decoder = new TextDecoder(); globalThis.foo = decoder.decode(buffer);"
        )?;
        assert_eq!("hello", ctx.global_object()?.get_property("foo")?.as_str()?);

        Ok(())
    }
}
