#![no_main]

use anyhow::Result;
use arbitrary_json::ArbitraryValue;
use javy::{
    from_js_error,
    quickjs::{Error, String as JSString},
    Config, Runtime,
};
use libfuzzer_sys::fuzz_target;
use std::sync::Once;

static JSON_PROGRAM: &[u8] = include_bytes!("json.js");
static mut RT: Option<Runtime> = None;
static mut REF_RT: Option<Runtime> = None;
static SETUP: Once = Once::new();

fuzz_target!(|data: ArbitraryValue| {
    SETUP.call_once(|| {
        let mut config = Config::default();
        config
            .override_json_parse_and_stringify(true)
            .javy_json(true);

        unsafe {
            RT = Some(Runtime::new(std::mem::take(&mut config)).expect("Runtime to be created"));
            REF_RT =
                Some(Runtime::new(Config::default()).expect("Reference runtime to be created"));
        };
    });

    let _ = exec(&data);
});

fn exec(data: &ArbitraryValue) -> Result<()> {
    let rt = unsafe { RT.as_ref().unwrap() };
    let ref_rt = unsafe { REF_RT.as_ref().unwrap() };
    let mut output: Option<String> = None;
    let mut ref_output: Option<String> = None;

    rt.context().with(|cx| {
        let globals = cx.globals();
        globals.set("INPUT", JSString::from_str(cx.clone(), &data.to_string())?)?;

        let result: Result<(), _> = cx.eval(JSON_PROGRAM);

        if let Err(e) = result {
            panic!("{}\n{}", from_js_error(cx.clone(), e), **data,);
        }

        let result: String = globals.get("OUTPUT")?;
        output = serde_json::from_str(&result).ok();

        Ok::<(), Error>(())
    })?;

    ref_rt.context().with(|cx| {
        let globals = cx.globals();
        globals.set("INPUT", JSString::from_str(cx.clone(), &data.to_string())?)?;

        let result: Result<(), _> = cx.eval(JSON_PROGRAM);

        if let Err(e) = result {
            panic!("{}\n{}", from_js_error(cx.clone(), e), **data);
        }

        let result: String = globals.get("OUTPUT")?;
        ref_output = serde_json::from_str(&result).ok();

        Ok::<(), Error>(())
    })?;

    assert_eq!(output, ref_output);

    Ok(())
}
