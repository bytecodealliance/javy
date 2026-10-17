#[cfg(feature = "json")]
use anyhow::Result;
#[cfg(feature = "json")]
use javy::{Config, Runtime, quickjs::context::EvalOptions};
use javy::{
    quickjs::{Context, Runtime as QuickJSRuntime, String as JSString},
    to_string_lossy,
};

#[cfg(feature = "json")]
#[test]
fn string_keys_and_ref_counting() -> Result<()> {
    let mut config = Config::default();
    config.simd_json_builtins(true);

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

#[cfg(feature = "json")]
#[test]
fn json_stringify_cycle_checks() -> Result<()> {
    let mut config = Config::default();
    config.simd_json_builtins(true);

    let source = include_bytes!("stringify_cycle.js");
    let rt = Runtime::new(config)?;

    rt.context().with(|this| {
        let _: () = this
            .eval_with_options(*source, EvalOptions::default())
            .inspect_err(|e| println!("{e}"))
            .expect("source evaluation to succeed");
    });

    Ok(())
}

#[test]
fn to_string_lossy_frees_engine_cstring() -> anyhow::Result<()> {
    let runtime = QuickJSRuntime::new()?;
    let context = Context::full(&runtime)?;

    let baseline = runtime.memory_usage().malloc_size;
    context.with(|this| {
        // A long string ending in a lone surrogate takes the lossy path.
        let string: JSString = this.eval(r#""a".repeat(65536) + "\uD800""#)?;
        for _ in 0..64 {
            let error = string
                .to_string()
                .expect_err("conversion should fail on the lone surrogate");
            let lossy = to_string_lossy(&this, &string, error)?;
            assert!(lossy.ends_with(char::REPLACEMENT_CHARACTER));
        }
        Ok::<_, anyhow::Error>(())
    })?;

    // Each call makes the engine allocate a ~192 KiB UTF-8 copy of the string.
    // The copies are reference counted, so if `to_string_lossy` does not free
    // them, not even a GC pass reclaims them and the ~12 MiB they amount to
    // here stays allocated.
    runtime.run_gc();
    let growth = runtime.memory_usage().malloc_size - baseline;
    assert!(
        growth < 1024 * 1024,
        "engine heap grew by {growth} bytes; the C strings were not freed"
    );
    Ok(())
}
