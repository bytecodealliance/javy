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

    // let crypto_obj = Object::nw(cx)?;
    let crypto_obj = Object::new(this.clone())?;

    crypto_obj.set(
        "createHmac",
        Function::new(this.clone(), |this, args| {
            let (this, args) = hold_and_release!(this, args);
            hmac_sha256(hold!(this.clone(), args)).map_err(|e| to_js_error(this, e))
        }),
    )?;

    globals.set("crypto", crypto_obj)?;

    Ok::<_, Error>(())
}
/// hmac_sha256 applies the HMAC algorithm for signing.
/// Arg[0] - algorithm (only supports sha256 today)
/// Arg[1] - secret key
/// returns - Hmac object
fn hmac_sha256(args: Args<'_>) -> Result<Class<HmacClass>> {
    let (ctx, args) = args.release();

    if args.len() != 2 {
        bail!("Wrong number of arguments. Expected 2. Got {}", args.len());
    }

    let js_string_algo = val_to_string(&ctx, args[0].clone())?;
    let js_string_secret = val_to_string(&ctx, args[1].clone())?;
    
    if js_string_algo != "sha256" {
        bail!("Argument 1: only sha256 supported.");
    }

    return Ok(
        Class::instance(
            ctx.clone(),
            HmacClass{
                algorithm: js_string_algo.clone(), 
                key: js_string_secret.clone(),
                message: JSString::from_str(ctx, "").unwrap(),
            }
        ).unwrap()
    );

    // prior method
    // Pass it to JavaScript

    // mac.update(js_string_message.as_bytes());

    // let result = mac.finalize();
    // let code_bytes = result.into_bytes();
    // let code: String = format!("{code_bytes:x}");
    // let js_string = JSString::from_str(cx, &code);
    // Ok(Value::from_string(js_string?))
}



fn hmac_sha256_result(secret: String, message: String) -> Result<String> {
    type HmacSha256 = Hmac<Sha256>;
    let mut hmac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    hmac.update(message.as_bytes());
    let result = hmac.finalize();
    let code_bytes = result.into_bytes();
    let code: String = format!("{code_bytes:x}");
    return Ok(code);
}

#[derive(rquickjs_macro::Trace)]
#[rquickjs_macro::class(rename_all = "camelCase")]
pub struct HmacClass<'js> {
    algorithm: String,
    key: String,

    #[qjs(get, set)]
    message: JSString<'js>,
}

#[rquickjs_macro::methods]
impl<'js> HmacClass<'js> {
    #[qjs()]
    pub fn digest(&self, type_of_digest: JSString<'js>) -> Result<Value<'js>, JsError> {
        let ctx = self.message.ctx();
        let js_type_of_digest = type_of_digest.to_string()?;
        if js_type_of_digest != "hex" {
            return Err(rquickjs::Exception::throw_type(&ctx.clone(), "digest type must be hex"));
        }

        let js_string_message = val_to_string(&ctx, self.message.clone().into()).map_err(|e| to_js_error(ctx.clone(), e));
        if js_string_message.is_err() {
            return Err(rquickjs::Exception::throw_type(&ctx.clone(), "failed val_to_string"));
        }
        let code = hmac_sha256_result(self.key.clone(), js_string_message?).map_err(|e| to_js_error(ctx.clone(), e));
        if code.is_err() {
            return Err(rquickjs::Exception::throw_type(&ctx.clone(), "failed hmac_sha256_result"));
        }
        let js_string = JSString::from_str(ctx.clone(), &code?);
        // let js_string = JSString::from_str(ctx.clone(), "hello world");
        if js_string.is_err() {
            return Err(rquickjs::Exception::throw_type(&ctx.clone(), "failed to convert string"));
        }
        Ok(Value::from_string(js_string?))
    }
}

#[cfg(test)]
mod tests {
    use crate::{quickjs::Value, Config, Runtime};
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
                    hmac.message = "input message";
                    hmac.digest("hex") === expectedHex;
            "#,
            )?;
            assert!(result.as_bool().unwrap());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}
