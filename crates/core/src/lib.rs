mod engine;
mod js_binding;
mod serialize;
mod transcode;

use js_binding::{context::Context, value::Value};

use once_cell::sync::OnceCell;
use std::io::{self, Read};
use transcode::{transcode_input, transcode_output};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut ENTRYPOINT: (OnceCell<Value>, OnceCell<Value>) = (OnceCell::new(), OnceCell::new());
static SCRIPT_NAME: &str = "script.js";

pub trait InputProvider<'de> {
    fn input() -> Result<Box<dyn serde::de::Deserializer<'de, Error = dyn serde::de::StdError + 'de>>, Box<dyn std::error::Error>>;
}

pub type OutputProvider = dyn Fn(crate::serialize::de::Deserializer) -> Result<(), Box<dyn std::error::Error>>;

struct ShopifyInputProvider {}

impl<'de> InputProvider<'de> for ShopifyInputProvider {
    fn input() -> Result<Box<dyn serde::de::Deserializer<'de, Error = dyn serde::de::StdError + 'de>>, Box<dyn std::error::Error>> {
        let input_bytes = engine::load()?;
        Ok(Box::new(rmp_serde::Deserializer::from_read_ref(input_bytes)));
    }
}



// TODO
//
// AOT validations:
//  1. Ensure that the required exports are present
//  2. If not present just evaluate the top level statement (?)

// use serde_javy::{to_value, from_value};

// fn to_value(json, Deserializer(JSON), serde_javy::Serializer) -> Result<Value>;
// fn from_value(value, Serializer(JSON)) -> Result<T>;

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        let mut context = Context::default();
        context.register_globals(io::stdout()).unwrap();

        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents).unwrap();

        let _ = context.eval_global(SCRIPT_NAME, &contents).unwrap();
        let global = context.global_object().unwrap();
        let shopify = global.get_property("Shopify").unwrap();
        let main = shopify.get_property("main").unwrap();

        JS_CONTEXT.set(context).unwrap();
        ENTRYPOINT.0.set(shopify).unwrap();
        ENTRYPOINT.1.set(main).unwrap();
    }
}

#[export_name = "shopify_main"]
pub extern "C" fn run() {
    unsafe {
        let context = JS_CONTEXT.get().unwrap();
        let shopify = ENTRYPOINT.0.get().unwrap();
        let main = ENTRYPOINT.1.get().unwrap();

        let input_provider = ShopifyInputProvider {};
        let deserializer = input_provider().expect("Did not read input");
        let mut serializer = crate::serialize::ser::Serializer::from_context(&context)?;
        serde_transcode::transcode(&mut deserializer, &mut serializer)?;
        let input_value: Value = serializer.value;

        // let input_bytes = engine::load().expect("Couldn't load input");

        // let input_value = transcode_input(&context, &input_bytes).unwrap();
        let output_value = main.call(&shopify, &[input_value]);

        if output_value.is_err() {
            panic!("{}", output_value.unwrap_err().to_string());
        }

        // let output = transcode_output(output_value.unwrap()).unwrap();
        // engine::store(&output).expect("Couldn't store output");

        let mut deserializer = crate::serialize::de::Deserializer::from(output_value.unwrap());
        let output_provider: OutputProvider = |deserializer: crate::serialize::de::Deserializer| {
            let mut output = Vec::new();
            let mut serializer = rmp_serde::Serializer::new(&mut output);
            serde_transcode::transcode(&mut deserializer, &mut serializer)?;
            engine::store(&output)
        };
        output_provider(deserializer).expect("Did not write output");
    }
}
