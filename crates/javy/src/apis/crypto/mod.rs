use crate::quickjs::{context::Intrinsic, qjs, Class, Ctx, Function, Object, String as JSString, Value};
use rquickjs::Error as JsError;
use crate::{hold, hold_and_release, to_js_error, val_to_string, Args};
use anyhow::{bail, Error, Result};

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// An implemetation of crypto APIs to optimize fuel.
/// Currently, hmacSHA256 is the only function implemented.
pub struct Crypto;

impl Intrinsic for Crypto {
    unsafe fn add_intrinsic(ctx: std::ptr::NonNull<qjs::JSContext>) {
        register(Ctx::from_raw(ctx)).expect("`Crypto` APIs to succeed")
    }
}

fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();

    let crypto_obj = Object::new(this.clone())?;

    crypto_obj.set(
        "createHmac",
        Function::new(this.clone(), |this, args| {
            let (this, args) = hold_and_release!(this, args);
            hmac_sha256_obj(hold!(this.clone(), args)).map_err(|e| to_js_error(this, e))
        }),
    )?;

    globals.set("crypto", crypto_obj)?;

    Ok::<_, Error>(())
}

/// hmac_sha256_obj creates the HMAC object
/// Arg[0] - algorithm (only supports sha256 today)
/// Arg[1] - secret key
/// returns - Hmac object
fn hmac_sha256_obj(args: Args<'_>) -> Result<Class<HmacClass>> {
    let (ctx, args) = args.release();

    if args.len() != 2 {
        bail!("Wrong number of arguments. Expected 2. Got {}", args.len());
    }

    let algo = val_to_string(&ctx, args[0].clone())?;
    let key = val_to_string(&ctx, args[1].clone())?;
    
    if algo != "sha256" {
        bail!("Argument 1: only sha256 supported.");
    }

    return Ok(
        Class::instance(
            ctx.clone(),
            HmacClass{
                algorithm: algo.clone(),
                key: key.clone(),
                message: JSString::from_str(ctx, "").unwrap(),
            }
        ).unwrap()
    );
}

/// hmac_sha256_result applies the HMAC sha256 algorithm for signing.
fn hmac_sha256_result(secret: String, message: String) -> Result<String> {
    type HmacSha256 = Hmac<Sha256>;
    let mut hmac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(message.as_bytes());
    let result = hmac.finalize();
    let code_bytes = result.into_bytes();
    let code: String = format!("{code_bytes:x}");
    Ok(code)
}

#[derive(rquickjs_macro::Trace)]
#[rquickjs_macro::class(rename_all = "camelCase")]
pub struct HmacClass<'js> {
    algorithm: String,
    key: String,
    message: JSString<'js>,
}

#[rquickjs_macro::methods]
impl<'js> HmacClass<'js> {
    #[qjs()]
    pub fn digest(&self, js_type_of_digest: JSString<'js>) -> Result<Value<'js>, JsError> {
        let ctx = self.message.ctx();

        // Convert JSString to Rust String
        let type_of_digest = js_type_of_digest.to_string()
            .map_err(|e| rquickjs::Exception::throw_type(ctx, &format!("Failed to convert type_of_digest to string: {}", e)))?;

        if type_of_digest != "hex" {
            return Err(rquickjs::Exception::throw_type(ctx, "digest type must be 'hex'"));
        }

        // Convert message to Rust String
        let string_message = val_to_string(ctx, self.message.clone().into())
            .map_err(|e| rquickjs::Exception::throw_type(ctx, &format!("Failed to convert message to string: {}", e)))?;

        // Compute HMAC
        let string_digest = hmac_sha256_result(self.key.clone(), string_message)
            .map_err(|e| rquickjs::Exception::throw_type(ctx, &format!("Failed to compute HMAC: {}", e)))?;

        // Convert result to JSString
        let js_string_digest = JSString::from_str(ctx.clone(), &string_digest)
            .map_err(|e| rquickjs::Exception::throw_type(ctx, &format!("Failed to convert result to JSString: {}", e)))?;

        Ok(Value::from_string(js_string_digest))
    }

    #[qjs()]
    pub fn update(&mut self, js_v: JSString<'js>) {
        let ctx = self.message.ctx();
        let mut string_message = val_to_string(ctx, self.message.clone().into())
          .map_err(|e| rquickjs::Exception::throw_type(ctx, &format!("Failed to convert message to string: {}", e))).unwrap();

        let v = val_to_string(ctx, js_v.clone().into())
          .map_err(|e| rquickjs::Exception::throw_type(ctx, &format!("Failed to convert update input to string: {}", e))).unwrap();

          string_message.push_str(&v);
        self.message = JSString::from_str(ctx.clone(), &string_message).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::{from_js_error, quickjs::Value, Config, Runtime};
    use anyhow::{Error, Result};

    #[test]
    fn test_crypto_digest() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result: Value<'_> = this.eval(
                r#"
                    let expectedHex = "97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9";
                    let hmac = crypto.createHmac("sha256", "my secret and secure key");
                    hmac.update("input message");
                    hmac.digest("hex") === expectedHex;
            "#,
            )?;
            assert!(result.as_bool().unwrap());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_crypto_disabled_by_default() -> Result<()> {
        let runtime = Runtime::new(Config::default())?;

        runtime.context().with(|this| {
            let result= this.eval::<Value<'_>, _>(
                r#"
                    crypto.createHmac("sha256", "hello world");
            "#,
            );
            assert!(result.is_err());
            let e = result.map_err(|e| from_js_error(this.clone(), e)).unwrap_err();
            assert_eq!("Error:2:21 'crypto' is not defined\n    at <eval> (eval_script:2:21)\n", e.to_string());
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
            let result: Value<'_> = this.eval(
                r#"
                    // matched tested behavior in node v18
                    let expectedHex = "c06ae855290abd8f397af6975e9c2f72fe27a90a3e0f0bb73b4f991567501980";
                    let hmac = crypto.createHmac("sha256", "\uD800\uD800\uD800\uD800\uD800");
                    hmac.update("\uD800\uD800\uD800\uD800\uD800");
                    let result = hmac.digest("hex");
                    console.log(result);
                    console.log("Match?", result === expectedHex);
                    result === expectedHex;
            "#,
            )?;
            assert!(result.as_bool().unwrap());
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
                    crypto.createHmac("not-sha", "my secret and secure key");
            "#,
            );
            assert!(result.is_err());
            let e = result.map_err(|e| from_js_error(this.clone(), e)).unwrap_err();
            assert_eq!("Error:2:28 Argument 1: only sha256 supported.\n    at <eval> (eval_script:2:28)\n", e.to_string());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_not_hex_digest_errors() -> Result<()> {
        let mut config = Config::default();
        config.crypto(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result= this.eval::<Value<'_>, _>(
                r#"
                    let hmac = crypto.createHmac("sha256", "my secret and secure key");
                    hmac.update("input message");
                    hmac.digest("base64");
            "#,
            );
            assert!(result.is_err());
            let e = result.map_err(|e| from_js_error(this.clone(), e)).unwrap_err();
            assert_eq!("Error:4:26 digest type must be 'hex'\n    at <eval> (eval_script:4:26)\n", e.to_string());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}
