use std::{borrow::Cow, str};

use anyhow::anyhow;
use javy::quickjs::{JSContextRef, JSError, JSValueRef};

use crate::JSApiSet;

pub(crate) struct TextEncoding {}

impl TextEncoding {
    pub(crate) fn new() -> Self {
        TextEncoding {}
    }
}

impl JSApiSet for TextEncoding {
    fn register(&self, context: &JSContextRef, _config: &crate::APIConfig) -> anyhow::Result<()> {
        let global = context.global_object()?;

        // let mut javy_object = global.get_property("Javy")?;
        // if javy_object.is_undefined() {
        //     javy_object = context.object_value()?;
        //     global.set_property("Javy", javy_object)?;
        // }

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
}

fn decode_utf8_buffer_to_js_string(
) -> impl FnMut(&JSContextRef, &JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValueRef> {
    move |ctx: &JSContextRef, _this: &JSValueRef, args: &[JSValueRef]| {
        if args.len() != 5 {
            return Err(anyhow!("Expecting 5 arguments, received {}", args.len()));
        }

        let buffer = args[0].as_bytes()?;
        let byte_offset = {
            let byte_offset_val = &args[1];
            if !byte_offset_val.is_repr_as_i32() {
                return Err(anyhow!("byte_offset must be an u32"));
            }
            byte_offset_val.as_u32_unchecked()
        }
        .try_into()?;
        let byte_length: usize = {
            let byte_length_val = &args[2];
            if !byte_length_val.is_repr_as_i32() {
                return Err(anyhow!("byte_length must be an u32"));
            }
            byte_length_val.as_u32_unchecked()
        }
        .try_into()?;
        let fatal = args[3].as_bool()?;
        let ignore_bom = args[4].as_bool()?;

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
        ctx.value_from_str(&str)
    }
}

fn encode_js_string_to_utf8_buffer(
) -> impl FnMut(&JSContextRef, &JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValueRef> {
    move |ctx: &JSContextRef, _this: &JSValueRef, args: &[JSValueRef]| {
        if args.len() != 1 {
            return Err(anyhow!("Expecting 1 argument, got {}", args.len()));
        }

        let js_string = args[0].as_str_lossy();
        ctx.array_buffer_value(js_string.as_bytes())
    }
}
