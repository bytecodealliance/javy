use crate::quickjs::{context::Intrinsic, qjs, Ctx, Object, String as JSString, Value, Function};
use crate::{
    hold, hold_and_release, val_to_string,
    to_js_error, Args
};
use anyhow::{bail, Error, Result};

use sha2::Sha256;
use hmac::{Hmac, Mac};

/// An implemetation of crypto APIs to optimize fuel.
/// Currently, hmacSHA256 is the only function implemented.
pub struct Cryptox;

impl Intrinsic for Cryptox {
    unsafe fn add_intrinsic(ctx: std::ptr::NonNull<qjs::JSContext>) {
        register(Ctx::from_raw(ctx)).expect("`Cryptox` APIs to succeed")
    }
}

fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();

    // let crypto_obj = Object::new(cx)?;
    let crypto_obj = Object::new(this.clone())?;

    crypto_obj.set(
        "hmacSHA256",
        Function::new(this.clone(), |this, args| {
            let (this, args) = hold_and_release!(this, args);
            hmac_sha256(hold!(this.clone(), args)).map_err(|e| to_js_error(this, e))
        }),
    )?;

    globals.set("cryptox", crypto_obj)?;

    Ok::<_, Error>(())
}
/// hmac_sha256 applies the HMAC algorithm using sha256 for hashing.
/// Arg[0] - secret
/// Arg[1] - message
/// returns - hex encoded string of hmac.
fn hmac_sha256(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, args) = args.release();

    if args.len() != 2 {
        bail!("Wrong number of arguments. Expected 2. Got {}", args.len());
    }

    let js_string_secret = val_to_string(&cx, args[0].clone())?;
    let js_string_message = val_to_string(&cx, args[1].clone())?;

    /// Create alias for HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(&js_string_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&js_string_message.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    let code : String = format!("{code_bytes:x}");
    let js_string = JSString::from_str(cx, &code);
    Ok(Value::from_string(js_string?))
}

#[cfg(test)]
mod tests {
    use crate::{quickjs::Value, Config, Runtime};
    use anyhow::{Error, Result};

    #[test]
    fn test_text_encoder_decoder() -> Result<()> {
        let mut config = Config::default();
        config.javy_cryptox(true);
        let runtime = Runtime::new(config)?;

        runtime.context().with(|this| {
            let result: Value<'_> = this.eval(
                r#"
                    let expectedHex = "97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9";
                    let result = cryptox.hmacSHA256("my secret and secure key", "input message");
                    expectedHex === result;
            "#,
            )?;

            assert!(result.as_bool().unwrap());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}