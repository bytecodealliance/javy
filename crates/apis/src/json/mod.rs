use anyhow::Result;
use std::io::Read;

use javy::{json, Runtime};
use quickjs_wasm_rs::from_qjs_value;

use crate::{APIConfig, JSApiSet};

pub(super) struct Json;

impl JSApiSet for Json {
    fn register(&self, runtime: &Runtime, _config: &APIConfig) -> Result<()> {
        let context = runtime.context();
        let global = context.global_object()?;
        global.set_property(
            "__javy_json_parse",
            context.wrap_callback(|cx, _this_arg, _| {
                let mut fd = std::io::stdin();
                let mut bytes = Vec::new();
                fd.read_to_end(&mut bytes).unwrap();
                let v = json::transcode_input(&cx, &bytes)?;

                from_qjs_value(v)
            })?,
        )?;
        context.eval_global("json.js", include_str!("./json.js"))?;
        Ok(())
    }
}
