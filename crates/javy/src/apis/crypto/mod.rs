use crate::quickjs::{context::Intrinsic, qjs, Class, Ctx, Function, Object, String as JSString, Value};

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
    
    // if (js_algo_secret != "sha256") {
    //     bail!("Argument 1: only sha256 supported.");
    // }

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

    // let cls = Class::instance(
    //     ctx.clone(),
    //     TestClass {
    //         inner_object: Object::new(ctx.clone()).unwrap(),
    //         some_value: 1,
    //         another_value: 2,
    //     },
    // )
    // .unwrap();

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
impl <'_>HmacClass<'_> {
    #[qjs(get, rename = "digest")]
    pub fn digest(&self, type_of_digest: JSString<'_>) -> Result<Value<'_>> {
        let ctx = self.message.ctx();

        let js_string_message = val_to_string(&ctx, self.message.clone().into()).unwrap();

        // return hmac_sha256_result(self.key.clone(), js_string_message).unwrap();
        let js_string = JSString::from_str(ctx.clone(), "hello world");

        // fails with:
        // lifetime may not live long enough
        // requirement occurs because of the type `rquickjs::Value<'_>`, which makes the generic argument `'_` invariant
        // the struct `rquickjs::Value<'js>` is invariant over the parameter `'js`
        // see <https://doc.rust-lang.org/nomicon/subtyping.html> for more information about variancerustcClick for full compiler diagnostic
        // mod.rs(111, 19): let's call the lifetime of this reference `'1`
        // mod.rs(111, 19): has type `&crypto::HmacClass<'2>`
        Ok(Value::from_string(js_string?))
    }
}

#[derive(rquickjs_macro::Trace)]
#[rquickjs_macro::class(rename_all = "camelCase")]
pub struct TestClass<'js> {
    /// These attribute make the accessible from JavaScript with getters and setters.
    /// As we used `rename_all = "camelCase"` in the attribute it will be called `innerObject`
    /// on the JavaScript side.
    #[qjs(get, set)]
    inner_object: Object<'js>,

    /// This works for any value which implements `IntoJs` and `FromJs` and is clonable.
    #[qjs(get, set)]
    some_value: u32,
    /// Make a field enumerable.
    #[qjs(get, set, enumerable)]
    another_value: u32,
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
                    // let expectedHex = "97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9";
                    // let result = crypto.hmacSHA256("my secret and secure key", "input message");
                    // expectedHex === result;
                    let hmac = crypto.createHmac("sha256", "my secret and secure key");
                    hmac.message = "input message";
                    // hmac.digest("hex") === expectedHex;
                    hmac.message === "input message";

                    hmac.digest("hex");
            "#,
            )?;

            // assert!(result.as_bool().unwrap());
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
}
