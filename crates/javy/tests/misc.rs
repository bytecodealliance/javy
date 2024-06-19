use anyhow::Result;
use javy::{quickjs::context::EvalOptions, Config, Runtime};

#[cfg(feature = "json")]
#[test]
fn string_keys_and_ref_counting() -> Result<()> {
    let mut config = Config::default();
    config.override_json_parse_and_stringify(true);

    let source = include_bytes!("string_keys_and_ref_counting.js");
    let rt = Runtime::new(config)?;

    rt.context().with(|this| {
        let _: () = this
            .eval_with_options(*source, EvalOptions::default())
            .inspect_err(|e| println!("{e}"))
            .expect("source evaluation to succeed");
    });

    Ok(())
}
