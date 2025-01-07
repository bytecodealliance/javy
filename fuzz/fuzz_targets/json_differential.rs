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
        let mut ref_config = Config::default();
        setup_config(&mut ref_config);

        let mut config = Config::default();
        setup_config(&mut config);
        config.simd_json_builtins(true);

        unsafe {
            RT = Some(Runtime::new(std::mem::take(&mut config)).expect("Runtime to be created"));
            REF_RT = Some(
                Runtime::new(std::mem::take(&mut ref_config))
                    .expect("Reference runtime to be created"),
            );
        };
    });

    let _ = exec(&data);
});

fn exec(data: &ArbitraryValue) -> Result<()> {
    let rt = unsafe { RT.as_ref().unwrap() };
    let ref_rt = unsafe { REF_RT.as_ref().unwrap() };
    let mut output: Option<String> = None;
    let mut ref_output: Option<String> = None;

    let input = data.to_string();
    let brace_count = input.chars().filter(|&c| c == '{').count();
    // Higher numbers of braces tends to cause a stack overflow (more braces
    // use more stack space).
    if brace_count > 350 {
        return Ok(());
    }

    rt.context().with(|cx| {
        let globals = cx.globals();

        globals.set("INPUT", JSString::from_str(cx.clone(), &input)?)?;

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
        globals.set("INPUT", JSString::from_str(cx.clone(), &input)?)?;

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

fn setup_config(config: &mut Config) {
    config
        // https://github.com/bellard/quickjs/blob/6e2e68fd0896957f92eb6c242a2e048c1ef3cae0/quickjs.c#L1644
        .gc_threshold(256 * 1024)
        // https://github.com/bellard/quickjs/blob/6e2e68fd0896957f92eb6c242a2e048c1ef3cae0/fuzz/fuzz_common.c#L33
        .memory_limit(0x4000000)
        // https://github.com/bellard/quickjs/blob/6e2e68fd0896957f92eb6c242a2e048c1ef3cae0/fuzz/fuzz_common.c#L35
        .max_stack_size(0x10000);
}
