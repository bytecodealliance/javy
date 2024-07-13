use crate::quickjs::{
    context::{EvalOptions, Intrinsic},
    qjs, Ctx, Function, String as JSString, Value,
};
use crate::{hold, hold_and_release, to_js_error, val_to_string, Args};
use anyhow::{bail, Error, Result};

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// A Winter CG compatible implementation of the Crypto API.
/// Currently, the following methods are implemented:
/// * `crypto.subtle.sign`, with HMAC sha256
pub struct Crypto;

impl Intrinsic for Crypto {
    unsafe fn add_intrinsic(ctx: std::ptr::NonNull<qjs::JSContext>) {
        register(Ctx::from_raw(ctx)).expect("`Crypto` APIs to succeed")
    }
}

fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();

    globals.set(
        "__javy_cryptoSubtleSign",
        Function::new(this.clone(), |this, args| {
            let (this, args) = hold_and_release!(this, args);
            hmac_sha256(hold!(this.clone(), args)).map_err(|e| to_js_error(this, e))
        }),
    )?;
    let mut opts = EvalOptions::default();
    opts.strict = false;
    this.eval_with_options(include_str!("crypto.js"), opts)?;

    Ok::<_, Error>(())
}

/// hmac_sha256 applies the HMAC algorithm using sha256 for hashing.
/// Arg[0] - secret
/// Arg[1] - message
/// returns - hex encoded string of hmac.
fn hmac_sha256(args: Args<'_>) -> Result<Value<'_>> {
    let (ctx, args) = args.release();

    if args.len() != 3 {
        bail!("Wrong number of arguments. Expected 3. Got {}", args.len());
    }

    let protocol = args[0].as_object();

    let js_protocol_name: Value = protocol.expect("protocol struct required").get("name")?;
    if val_to_string(&ctx, js_protocol_name.clone())? != "HMAC" {
        bail!("only name=HMAC supported");
    }

    let js_protocol_name: Value = protocol.expect("protocol struct required").get("hash")?;
    if val_to_string(&ctx, js_protocol_name.clone())? != "sha-256" {
        bail!("only hash=sha-256 supported");
    }
    let secret = val_to_string(&ctx, args[1].clone())?;
    let message = val_to_string(&ctx, args[2].clone())?;

    let string_digest = hmac_sha256_result(secret, message)?;
    let result = JSString::from_str(ctx.clone(), &string_digest)?;
    Ok(result.into())
}

/// hmac_sha256_result applies the HMAC sha256 algorithm for signing.
fn hmac_sha256_result(secret: String, message: String) -> Result<String> {
    type HmacSha256 = Hmac<Sha256>;
    let mut hmac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(message.as_bytes());
    let result = hmac.finalize();
    let code_bytes = result.into_bytes();
    let code: String = format!("{code_bytes:x}");
    Ok(code)
}

#[cfg(test)]
mod tests {
    use crate::{from_js_error, val_to_string, quickjs::Value, Config, Runtime};
    use anyhow::{Error, Result};

    #[test]
    fn test_crypto_digest_internal() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result = this.eval::<Value<'_>, _>(
                r#"
                    const __javy_cryptoSubtleSign = globalThis.__javy_cryptoSubtleSign;
                    //crypto.subtle.sign({name: "HMAC", hash: "sha-256"}, "my secret and secure key", "input message").then(function(sig) { result = sig });
                    __javy_cryptoSubtleSign({name: "HMAC", hash: "sha-256"}, "my secret and secure key", "input message");
            "#,
            );
            assert_eq!(val_to_string(&this, result.unwrap()).unwrap(), "97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9");
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_crypto_disabled_by_default() -> Result<()> {
        let runtime = Runtime::new(Config::default())?;

        runtime.context().with(|this| {
            let result = this.eval::<Value<'_>, _>(
                r#"
                    crypto.subtle;
            "#,
            );
            assert!(result.is_err());
            let e = result
                .map_err(|e| from_js_error(this.clone(), e))
                .unwrap_err();
            assert_eq!(
                "Error:2:21 'crypto' is not defined\n    at <eval> (eval_script:2:21)\n",
                e.to_string()
            );
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_crypto_digest_with_lossy_input() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result = this.eval::<Value<'_>, _>(
                r#"
                    // matched tested behavior in node v18
                    // result = crypto.subtle.sign({name: "HMAC", hash: "sha-256"}, "\uD800\uD800\uD800\uD800\uD800", "\uD800\uD800\uD800\uD800\uD800")
                    const __javy_cryptoSubtleSign = globalThis.__javy_cryptoSubtleSign;
                    __javy_cryptoSubtleSign({name: "HMAC", hash: "sha-256"}, "\uD800\uD800\uD800\uD800\uD800", "\uD800\uD800\uD800\uD800\uD800");
            "#,
            )?;
            assert_eq!(val_to_string(&this, result).unwrap(), "c06ae855290abd8f397af6975e9c2f72fe27a90a3e0f0bb73b4f991567501980");
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_crypto_undefined_methods_raise_not_a_function() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result= this.eval::<Value<'_>, _>(
                r#"
                    crypto.subtle.encrypt({name: "HMAC", hash: "sha-256"}, "my secret and secure key", "input message");
            "#,
            );
            assert!(result.is_err());
            let e = result.map_err(|e| from_js_error(this.clone(), e)).unwrap_err();
            assert_eq!("Error:2:35 not a function\n    at <eval> (eval_script:2:35)\n", e.to_string());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_not_hmac_algo_errors() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result= this.eval::<Value<'_>, _>(
                r#"
                    // let result = crypto.subtle.sign({name: "not-HMAC", hash: "not-sha-256"}, "my secret and secure key", "input message");
                    const __javy_cryptoSubtleSign = globalThis.__javy_cryptoSubtleSign;
                    __javy_cryptoSubtleSign({name: "not-HMAC", hash: "not-sha-256"}, "my secret and secure key", "input message");

            "#,
            );
            assert!(result.is_err());
            let e = result.map_err(|e| from_js_error(this.clone(), e)).unwrap_err();
            assert_eq!("Error:4:21 only name=HMAC supported\n    at <eval> (eval_script:4:21)\n", e.to_string());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_not_sha256_algo_errors() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result= this.eval::<Value<'_>, _>(
                r#"
                    // let result = crypto.subtle.sign({name: "HMAC", hash: "not-sha-256"}, "my secret and secure key", "input message");
                    const __javy_cryptoSubtleSign = globalThis.__javy_cryptoSubtleSign;
                    __javy_cryptoSubtleSign(this, "my secret and secure key", "input message");
            "#,
            );
            assert!(result.is_err());
            let e = result.map_err(|e| from_js_error(this.clone(), e)).unwrap_err();
            assert_eq!("Error:4:21 only name=HMAC supported\n    at <eval> (eval_script:4:21)\n", e.to_string());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}
