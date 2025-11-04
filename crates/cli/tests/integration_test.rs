use anyhow::{bail, Result};
use javy_runner::{Builder, Plugin, Runner, RunnerError, Source};
use std::{fs, io::Read, path::PathBuf, process::Command, str};
use wasmparser::Parser;
use wasmtime::{AsContextMut, Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

use javy_test_macros::javy_cli_test;

#[javy_cli_test]
fn test_empty(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("empty.js").build()?;

    let (_, _, fuel_consumed) = run(&mut runner, vec![]);
    assert_fuel_consumed_within_threshold(22_448, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_identity(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.build()?;

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 42);
    assert_eq!(42, output);
    assert_fuel_consumed_within_threshold(46_797, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_fib(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("fib.js").build()?;

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
    assert_fuel_consumed_within_threshold(64_681, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_recursive_fib(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("recursive-fib.js").build()?;

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
    assert_fuel_consumed_within_threshold(67_869, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_str(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("str.js").build()?;

    let (output, _, fuel_consumed) = run(&mut runner, "hello".into());
    assert_eq!("world".as_bytes(), output);
    assert_fuel_consumed_within_threshold(146_027, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_encoding(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("text-encoding.js").build()?;

    let (output, _, fuel_consumed) = run(&mut runner, "hello".into());
    assert_eq!("el".as_bytes(), output);
    assert_fuel_consumed_within_threshold(244_706, fuel_consumed);

    let (output, _, _) = run(&mut runner, "invalid".into());
    assert_eq!("true".as_bytes(), output);

    let (output, _, _) = run(&mut runner, "invalid_fatal".into());
    assert_eq!("The encoded data was not valid utf-8".as_bytes(), output);

    let (output, _, _) = run(&mut runner, "test".into());
    assert_eq!("test2".as_bytes(), output);
    Ok(())
}

#[javy_cli_test]
fn test_console_log(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("logging.js").build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, vec![]);
    assert_eq!(b"hello world from console.log\n".to_vec(), output);
    assert_eq!("hello world from console.error\n", logs.as_str());
    assert_fuel_consumed_within_threshold(34_983, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_using_plugin_with_static_build(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .plugin(Plugin::UserWasiP2)
        .input("plugin.js")
        .build()?;

    let result = runner.exec(vec![]);
    assert!(result.is_ok());

    Ok(())
}

#[javy_cli_test]
fn test_using_plugin_with_static_build_fails_with_runtime_config(
    builder: &mut Builder,
) -> Result<()> {
    let result = builder
        .plugin(Plugin::UserWasiP2)
        .simd_json_builtins(true)
        .build();
    let err = result.err().unwrap();
    assert!(err
        .to_string()
        .contains("Property simd-json-builtins is not supported for runtime configuration"));

    Ok(())
}

#[javy_cli_test]
fn test_using_invalid_plugin_with_static_build_fails(builder: &mut Builder) -> Result<()> {
    let result = builder.plugin(Plugin::InvalidUser).build();
    let err = result.err().unwrap();
    assert!(
        err.to_string()
            .contains("Could not process plugin: Using unsupported legacy plugin API"),
        "Expected a different string but got: {}",
        err,
    );
    Ok(())
}

#[javy_cli_test]
fn test_readme_script(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("readme.js").build()?;

    let (output, _, fuel_consumed) = run(&mut runner, r#"{ "n": 2, "bar": "baz" }"#.into());
    assert_eq!(r#"{"foo":3,"newBar":"baz!"}"#.as_bytes(), output);
    assert_fuel_consumed_within_threshold(254_503, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_promises_with_event_loop(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("promise.js").event_loop(true).build()?;

    let (output, _, _) = run(&mut runner, vec![]);
    assert_eq!("\"foo\"\"bar\"".as_bytes(), output);
    Ok(())
}

#[javy_cli_test]
fn test_promises_without_event_loop(builder: &mut Builder) -> Result<()> {
    use javy_runner::RunnerError;

    let mut runner = builder.input("promise.js").build()?;
    let res = runner.exec(vec![]);
    let err = res.err().unwrap().downcast::<RunnerError>().unwrap();
    assert!(err.stderr.contains("Pending jobs in the event queue."));

    Ok(())
}

#[javy_cli_test]
fn test_promise_top_level_await(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("top-level-await.js")
        .event_loop(true)
        .build()?;
    let (out, _, _) = run(&mut runner, vec![]);

    assert_eq!("bar", String::from_utf8(out)?);
    Ok(())
}

#[javy_cli_test]
fn test_exported_functions(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-fn.js")
        .wit("exported-fn.wit")
        .world("exported-fn")
        .build()?;
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "foo", vec![]);
    assert_eq!("Hello from top-level\nHello from foo\n", logs);
    assert_fuel_consumed_within_threshold(59_981, fuel_consumed);
    let (_, logs, _) = run_fn(&mut runner, "foo-bar", vec![]);
    assert_eq!("Hello from top-level\nHello from fooBar\n", logs);
    Ok(())
}

#[javy_cli_test]
fn test_exported_promises(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-promise-fn.js")
        .wit("exported-promise-fn.wit")
        .world("exported-promise-fn")
        .event_loop(true)
        .build()?;
    let (_, logs, _) = run_fn(&mut runner, "foo", vec![]);
    assert_eq!("Top-level\ninside foo\n", logs);
    Ok(())
}

#[javy_cli_test]
fn test_exported_functions_without_flag(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("exported-fn.js").build()?;
    let res = runner.exec_func("foo", vec![]);
    assert_eq!(
        "failed to find function export `foo`",
        res.err().unwrap().to_string()
    );
    Ok(())
}

#[javy_cli_test]
fn test_exported_function_without_semicolons(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-fn-no-semicolon.js")
        .wit("exported-fn-no-semicolon.wit")
        .world("exported-fn")
        .build()?;
    run_fn(&mut runner, "foo", vec![]);
    Ok(())
}

#[javy_cli_test]
fn test_producers_section_present(builder: &mut Builder) -> Result<()> {
    let runner = builder.input("readme.js").build()?;

    runner.assert_producers()
}

#[javy_cli_test]
fn test_error_handling(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("error.js").build()?;
    let result = runner.exec(vec![]);
    let err = result.err().unwrap().downcast::<RunnerError>().unwrap();

    let expected_log_output =
        "Error: error\n    at error (function.mjs:2:13)\n    at <anonymous> (function.mjs:5:1)\n\n";

    assert_eq!(expected_log_output, err.stderr);
    Ok(())
}

#[javy_cli_test]
fn test_same_module_outputs_different_random_result(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("random.js").build()?;
    let (output, _, _) = runner.exec(vec![]).unwrap();
    let (output2, _, _) = runner.exec(vec![]).unwrap();
    // In theory these could be equal with a correct implementation but it's very unlikely.
    assert!(output != output2);
    // Don't check fuel consumed because fuel consumed can be different from run to run. See
    // https://github.com/bytecodealliance/javy/issues/401 for investigating the cause.
    Ok(())
}

#[javy_cli_test]
fn test_exported_default_arrow_fn(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-default-arrow-fn.js")
        .wit("exported-default-arrow-fn.wit")
        .world("exported-arrow")
        .build()?;

    let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", vec![]);
    assert_eq!(logs, "42\n");
    assert_fuel_consumed_within_threshold(37_298, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_exported_default_fn(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-default-fn.js")
        .wit("exported-default-fn.wit")
        .world("exported-default")
        .build()?;
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", vec![]);
    assert_eq!(logs, "42\n");
    assert_fuel_consumed_within_threshold(39_147, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_string_normalize(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("normalize.js").build()?;
    let (_, logs, fuel_consumed) = run(&mut runner, vec![]);
    assert_eq!(logs, "true\n");
    assert_fuel_consumed_within_threshold(55_418, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_source_code_default(builder: &mut Builder) -> Result<()> {
    let runner = builder.build()?;
    let javy_source = runner
        .javy_source_custom_section()
        .expect("Should have javy_source custom section");
    let mut decompressor = brotli::Decompressor::new(javy_source, 4096);
    let mut javy_source = vec![];
    let res = decompressor.read_to_end(&mut javy_source);
    assert!(res.is_ok());
    Ok(())
}

#[javy_cli_test]
fn test_source_code_compressed(builder: &mut Builder) -> Result<()> {
    let runner = builder.source_code(Source::Compressed).build()?;
    let javy_source = runner
        .javy_source_custom_section()
        .expect("Should have javy_source custom section");
    let mut decompressor = brotli::Decompressor::new(javy_source, 4096);
    let mut javy_source = vec![];
    let res = decompressor.read_to_end(&mut javy_source);
    assert!(res.is_ok());
    Ok(())
}

#[javy_cli_test]
fn test_source_code_uncompressed(builder: &mut Builder) -> Result<()> {
    let runner = builder.source_code(Source::Uncompressed).build()?;
    let javy_source = runner
        .javy_source_custom_section()
        .expect("Should have javy_source custom section");
    let mut decompressor = brotli::Decompressor::new(javy_source, 4096);
    let mut javy_source = vec![];
    let res = decompressor.read_to_end(&mut javy_source);
    assert!(res.is_err());
    Ok(())
}

#[javy_cli_test]
fn test_source_code_omitted(builder: &mut Builder) -> Result<()> {
    let runner = builder.source_code(Source::Omitted).build()?;
    assert!(
        runner.javy_source_custom_section().is_none(),
        "Should not have a source code section"
    );
    Ok(())
}

#[test]
fn test_init_plugin() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().build_p1();
    let mut store = Store::new(&engine, wasi);

    let uninitialized_plugin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(
            std::path::Path::new("target")
                .join("wasm32-wasip2")
                .join("release")
                .join("plugin.wasm"),
        );

    // Check that plugin is in fact uninitialized at this point by seeing if it's a component.
    assert!(
        Parser::is_component(&fs::read(&uninitialized_plugin)?),
        "Expected Wasm file to be component but was not a component"
    );

    // Initialize the plugin.
    let output = Command::new(env!("CARGO_BIN_EXE_javy"))
        .arg("init-plugin")
        .arg(uninitialized_plugin.to_str().unwrap())
        .output()?;
    if !output.status.success() {
        bail!(
            "Running init-command failed with output {}",
            str::from_utf8(&output.stderr)?,
        );
    }
    let initialized_plugin = output.stdout;

    // Check the plugin is initialized and runs.
    let module = Module::new(&engine, &initialized_plugin)?;
    let instance = linker.instantiate(store.as_context_mut(), &module)?;
    // This should succeed because the runtime is initialized.
    instance
        .get_typed_func::<(i32, i32), i32>(store.as_context_mut(), "compile-src")?
        .call(store.as_context_mut(), (0, 0))?;
    Ok(())
}

fn run_with_u8s(r: &mut Runner, stdin: u8) -> (u8, String, u64) {
    let (output, logs, fuel_consumed) = run(r, stdin.to_le_bytes().into());
    assert_eq!(1, output.len());
    (output[0], logs, fuel_consumed)
}

fn run(r: &mut Runner, stdin: Vec<u8>) -> (Vec<u8>, String, u64) {
    run_fn(r, "_start", stdin)
}

fn run_fn(r: &mut Runner, func: &str, stdin: Vec<u8>) -> (Vec<u8>, String, u64) {
    let (output, logs, fuel_consumed) = r.exec_func(func, stdin).unwrap();
    let logs = String::from_utf8(logs).unwrap();
    (output, logs, fuel_consumed)
}

/// Used to detect any significant changes in the fuel consumption when making
/// changes in Javy.
///
/// A threshold is used here so that we can decide how much of a change is
/// acceptable. The threshold value needs to be sufficiently large enough to
/// account for fuel differences between different operating systems.
///
/// If the fuel_consumed is less than target_fuel, then great job decreasing the
/// fuel consumption! However, if the fuel_consumed is greater than target_fuel
/// and over the threshold, please consider if the changes are worth the
/// increase in fuel consumption.
fn assert_fuel_consumed_within_threshold(target_fuel: u64, fuel_consumed: u64) {
    let target_fuel = target_fuel as f64;
    let fuel_consumed = fuel_consumed as f64;
    let threshold = 3.0;
    let percentage_difference = ((fuel_consumed - target_fuel) / target_fuel).abs() * 100.0;

    assert!(
        percentage_difference <= threshold,
        "fuel_consumed ({fuel_consumed}) was not within {threshold:.2}% of the target_fuel value ({target_fuel})",
    );
}
