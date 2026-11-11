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
    Args, hold, json,
    quickjs::{
        Ctx, Exception, Function, Object, String as JSString, Value,
        atom::PredefinedAtom,
        function::This,
        prelude::{MutFn, Rest},
    },
    to_js_error, val_to_string,
};

use simd_json::{Error as SError, ErrorType};

use anyhow::{Result, anyhow, bail};
use std::{sync::OnceLock, time::SystemTime};

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
            // Fast path. `null` and numbers that round-trip through their
            // string form are treated as identity. NaN, +/-Infinity and -0 are
            // excluded: their string forms are either invalid JSON or parse to
            // a different value, so they take the regular path.
            if val.is_null() {
                return Ok(val);
            }
            if let Some(n) = val.as_number()
                && n.is_finite()
                && !(n == 0.0 && n.is_sign_negative())
            {
                return Ok(val);
            }

            if val.is_symbol() {
                bail!(Exception::throw_type(&this, "Expected string primitive"));
            }

            let input = val_to_string(&this, args[0].clone())?;

            // simd-json diverges from the spec in a few cases it accepts or
            // rejects, so defer to the built-in `JSON.parse` for those:
            // - A negative zero integer (`-0`) would be parsed as `0`.
            // - Anything simd-json rejects: out of range numbers (`1e400`),
            //   lone surrogates (`"\ud800"`), where the built-in parser either
            //   succeeds or throws its own error. Excessive nesting is the
            //   exception, see below.
            if contains_negative_zero_integer(input.as_bytes()) {
                return call_default_parse(&this, &input);
            }

            // simd-json unescapes strings in place, which can leave the
            // buffer holding invalid UTF-8, so parse from an owned byte
            // buffer rather than the `String`'s bytes.
            let mut bytes = input.clone().into_bytes();
            json::parse(this.clone(), &mut bytes).or_else(|original| {
                match original.downcast_ref::<SError>() {
                    // Don't fall back for excessive nesting: the built-in
                    // parser has no depth limit of its own on Wasm and would
                    // exhaust the call stack and trap.
                    Some(e) if matches!(e.error(), ErrorType::DepthLimitExceeded) => {
                        Err(anyhow!(Exception::throw_syntax(&this, &e.to_string())))
                    }
                    Some(_) => call_default_parse(&this, &input),
                    None => Err(original),
                }
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

/// Invokes the built-in QuickJS `JSON.parse` on an already stringified input.
fn call_default_parse<'js>(this: &Ctx<'js>, input: &str) -> Result<Value<'js>> {
    let default: Function = this.globals().get(DEFAULT_PARSE_KEY.get().unwrap())?;
    let input = JSString::from_str(this.clone(), input)?;
    default.call((input,)).map_err(|e| anyhow!(e))
}

/// Conservatively detects a `-0` that isn't followed by a digit, `.`, `e` or
/// `E`, i.e. a negative zero integer, which simd-json would parse as `0`.
/// May also match `-0` inside a string literal, which is harmless since the
/// built-in parser handles those correctly too.
fn contains_negative_zero_integer(bytes: &[u8]) -> bool {
    bytes
        .windows(3)
        .any(|w| w[0] == b'-' && w[1] == b'0' && !matches!(w[2], b'0'..=b'9' | b'.' | b'e' | b'E'))
        || bytes.ends_with(b"-0")
}

fn call_json_stringify(args: Args<'_>) -> Result<Value<'_>> {
    let (this, args) = args.release();

    match args.len() {
        0 => Ok(Value::new_undefined(this.clone())),
        1 => {
            let arg = args[0].clone();
            let val: Value = if arg.is_object() {
                if let Some(f) = get_to_json(&arg)? {
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

/// Looks up the `toJSON` property of an object, propagating any exception
/// thrown by a getter or proxy trap during the lookup.
fn get_to_json<'a>(value: &Value<'a>) -> Result<Option<Function<'a>>> {
    let Some(object) = value.as_object() else {
        return Ok(None);
    };
    Ok(object
        .get::<_, Value<'a>>(PredefinedAtom::ToJSON)?
        .into_function())
}

#[cfg(test)]
mod tests {
    use crate::{Config, Runtime};
    use anyhow::Result;

    fn runtime() -> Result<Runtime> {
        let mut config = Config::default();
        config.simd_json_builtins(true);
        Runtime::new(config)
    }

    #[test]
    fn test_parse_non_json_numbers_throw() -> Result<()> {
        let rt = runtime()?;
        rt.context().with(|cx| {
            for input in ["NaN", "Infinity", "-Infinity"] {
                let threw: bool = cx.eval(format!(
                    "try {{ JSON.parse({input}); false }} catch (e) {{ e instanceof SyntaxError }}"
                ))?;
                assert!(threw, "JSON.parse({input}) should throw a SyntaxError");
            }
            Ok::<_, anyhow::Error>(())
        })
    }

    #[test]
    fn test_parse_negative_zero_is_positive_zero() -> Result<()> {
        let rt = runtime()?;
        rt.context().with(|cx| {
            let is_pos_zero: bool = cx.eval("Object.is(JSON.parse(-0), 0)")?;
            assert!(is_pos_zero);
            Ok::<_, anyhow::Error>(())
        })
    }

    fn eval_bool(code: &str) -> Result<bool> {
        let rt = runtime()?;
        rt.context().with(|cx| Ok(cx.eval(code)?))
    }

    #[test]
    fn test_parse_negative_zero_string() -> Result<()> {
        assert!(eval_bool(
            r#"Object.is(JSON.parse("-0"), -0)
               && Object.is(JSON.parse(" -0 "), -0)
               && Object.is(JSON.parse("[-0]")[0], -0)
               && Object.is(JSON.parse('{"a":-0}').a, -0)
               && Object.is(JSON.parse("-0.0"), -0)
               && JSON.parse("-0.5") === -0.5
               && JSON.parse("-10") === -10
               && JSON.parse('"-0"') === "-0""#
        )?);
        Ok(())
    }

    #[test]
    fn test_parse_out_of_range_number_is_infinity() -> Result<()> {
        assert!(eval_bool(
            r#"JSON.parse("1e400") === Infinity && JSON.parse("-1e400") === -Infinity"#
        )?);
        Ok(())
    }

    #[test]
    fn test_parse_lone_surrogate() -> Result<()> {
        assert!(eval_bool(
            r#"JSON.parse('"\\ud800"').charCodeAt(0) === 0xd800"#
        )?);
        Ok(())
    }

    #[test]
    fn test_parse_deep_nesting_throws() -> Result<()> {
        // Must throw rather than recursing without bound, which would exhaust
        // the stack on Wasm.
        assert!(eval_bool(
            r#"try { JSON.parse("[".repeat(100000)); false } catch (e) { e instanceof SyntaxError }"#
        )?);
        Ok(())
    }

    #[test]
    fn test_parse_invalid_json_throws_syntax_error() -> Result<()> {
        assert!(eval_bool(
            r#"["", "{", "[1,]", "01", "tru"].every(s => {
                try { JSON.parse(s); return false } catch (e) { return e instanceof SyntaxError }
            })"#
        )?);
        Ok(())
    }

    #[test]
    fn test_parse_numbers_and_null_fast_path() -> Result<()> {
        let rt = runtime()?;
        rt.context().with(|cx| {
            let ok: bool = cx.eval(
                "JSON.parse(1) === 1 && JSON.parse(-1.5) === -1.5 && JSON.parse(0) === 0 && JSON.parse(null) === null",
            )?;
            assert!(ok);
            Ok::<_, anyhow::Error>(())
        })
    }
}
