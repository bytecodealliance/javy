use quickjs_sys as q;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
struct Rename<T>(#[serde(rename(deserialize = "camelCase"))] pub T);

pub fn prepare(context: &crate::Context, val: q::JSValue) -> Vec<u8> {
    let js_string = context.deserialize_string(val);
    let value: serde_json::Value = serde_json::from_str::<Rename<_>>(&js_string).unwrap().0;

    rmp_serde::to_vec_named(&value).unwrap()
}

