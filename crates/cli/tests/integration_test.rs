use anyhow::{bail, Result};
use javy_runner::{Builder, Plugin, Runner, RunnerError};
use std::{path::PathBuf, process::Command, str};
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::{AsContextMut, Engine, Linker, Module, Store};

use javy_test_macros::javy_cli_test;

#[javy_cli_test]
fn test_empty(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("empty.js").build()?;

    let (_, _, fuel_consumed) = run(&mut runner, &[]);
    assert_fuel_consumed_within_threshold(22_590, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_identity(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.build()?;

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 42);
    assert_eq!(42, output);
    assert_fuel_consumed_within_threshold(47_773, fuel_consumed);
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
    assert_fuel_consumed_within_threshold(69_306, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_str(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("str.js").build()?;

    let (output, _, fuel_consumed) = run(&mut runner, "hello".as_bytes());
    assert_eq!("world".as_bytes(), output);
    assert_fuel_consumed_within_threshold(146_027, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_encoding(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("text-encoding.js").build()?;

    let (output, _, fuel_consumed) = run(&mut runner, "hello".as_bytes());
    assert_eq!("el".as_bytes(), output);
    assert_fuel_consumed_within_threshold(258_197, fuel_consumed);

    let (output, _, _) = run(&mut runner, "invalid".as_bytes());
    assert_eq!("true".as_bytes(), output);

    let (output, _, _) = run(&mut runner, "invalid_fatal".as_bytes());
    assert_eq!("The encoded data was not valid utf-8".as_bytes(), output);

    let (output, _, _) = run(&mut runner, "test".as_bytes());
    assert_eq!("test2".as_bytes(), output);
    Ok(())
}

#[javy_cli_test(commands(not(Build)))]
fn test_logging_with_compile(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("logging.js").build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert!(output.is_empty());
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
    assert_fuel_consumed_within_threshold(36_071, fuel_consumed);
    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_logging_without_redirect(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("logging.js")
        .redirect_stdout_to_stderr(false)
        .build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert_eq!(b"hello world from console.log\n".to_vec(), output);
    assert_eq!("hello world from console.error\n", logs.as_str());
    assert_fuel_consumed_within_threshold(36_641, fuel_consumed);
    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_logging_with_redirect(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("logging.js")
        .redirect_stdout_to_stderr(true)
        .build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert!(output.is_empty());
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
    assert_fuel_consumed_within_threshold(35_007, fuel_consumed);
    Ok(())
}

#[javy_cli_test(commands(not(Compile)), root = "tests/dynamic-linking-scripts")]
fn test_javy_json_enabled(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("javy-json-id.js").build()?;

    let input = "{\"x\":5}";
    let (output, logs, _) = run(&mut runner, input.as_bytes());

    assert_eq!(logs, "undefined\n");
    assert_eq!(String::from_utf8(output)?, input);

    Ok(())
}

#[javy_cli_test(commands(not(Compile)), root = "tests/dynamic-linking-scripts")]
fn test_javy_json_disabled(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("javy-json-id.js")
        .simd_json_builtins(false)
        .build()?;

    let result = runner.exec(&[]);
    assert!(result.is_err());

    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_using_plugin_with_static_build(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.plugin(Plugin::User).input("plugin.js").build()?;

    let result = runner.exec(&[]);
    assert!(result.is_ok());

    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_using_plugin_with_static_build_fails_with_runtime_config(
    builder: &mut Builder,
) -> Result<()> {
    let result = builder
        .plugin(Plugin::User)
        .simd_json_builtins(true)
        .build();
    let err = result.err().unwrap();
    assert!(err
        .to_string()
        .contains("Property simd-json-builtins is not supported for runtime configuration"));

    Ok(())
}

#[javy_cli_test]
fn test_readme_script(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("readme.js").build()?;

    let (output, _, fuel_consumed) = run(&mut runner, r#"{ "n": 2, "bar": "baz" }"#.as_bytes());
    assert_eq!(r#"{"foo":3,"newBar":"baz!"}"#.as_bytes(), output);
    assert_fuel_consumed_within_threshold(254_503, fuel_consumed);
    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_promises_with_event_loop(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("promise.js").event_loop(true).build()?;

    let (output, _, _) = run(&mut runner, &[]);
    assert_eq!("\"foo\"\"bar\"".as_bytes(), output);
    Ok(())
}

#[javy_cli_test]
fn test_promises_without_event_loop(builder: &mut Builder) -> Result<()> {
    use javy_runner::RunnerError;

    let mut runner = builder.input("promise.js").build()?;
    let res = runner.exec(&[]);
    let err = res.err().unwrap().downcast::<RunnerError>().unwrap();
    assert!(err.stderr.contains("Pending jobs in the event queue."));

    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_promise_top_level_await(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("top-level-await.js")
        .event_loop(true)
        .build()?;
    let (out, _, _) = run(&mut runner, &[]);

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
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "foo", &[]);
    assert_eq!("Hello from top-level\nHello from foo\n", logs);
    assert_fuel_consumed_within_threshold(61_404, fuel_consumed);
    let (_, logs, _) = run_fn(&mut runner, "foo-bar", &[]);
    assert_eq!("Hello from top-level\nHello from fooBar\n", logs);
    Ok(())
}

#[javy_cli_test(commands(not(Compile)))]
fn test_exported_promises(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-promise-fn.js")
        .wit("exported-promise-fn.wit")
        .world("exported-promise-fn")
        .event_loop(true)
        .build()?;
    let (_, logs, _) = run_fn(&mut runner, "foo", &[]);
    assert_eq!("Top-level\ninside foo\n", logs);
    Ok(())
}

#[javy_cli_test]
fn test_exported_functions_without_flag(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("exported-fn.js").build()?;
    let res = runner.exec_func("foo", &[]);
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
    run_fn(&mut runner, "foo", &[]);
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
    let result = runner.exec(&[]);
    let err = result.err().unwrap().downcast::<RunnerError>().unwrap();

    let expected_log_output = "Error:2:9 error\n    at error (function.mjs:2:9)\n    at <anonymous> (function.mjs:5:1)\n\n";

    assert_eq!(expected_log_output, err.stderr);
    Ok(())
}

#[javy_cli_test]
fn test_same_module_outputs_different_random_result(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("random.js").build()?;
    let (output, _, _) = runner.exec(&[]).unwrap();
    let (output2, _, _) = runner.exec(&[]).unwrap();
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

    let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", &[]);
    assert_eq!(logs, "42\n");
    assert_fuel_consumed_within_threshold(39_004, fuel_consumed);
    Ok(())
}

#[javy_cli_test]
fn test_exported_default_fn(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("exported-default-fn.js")
        .wit("exported-default-fn.wit")
        .world("exported-default")
        .build()?;
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", &[]);
    assert_eq!(logs, "42\n");
    assert_fuel_consumed_within_threshold(39_951, fuel_consumed);
    Ok(())
}

#[test]
fn test_init_plugin() -> Result<()> {
    // This test works by trying to call the `compile_src` function on the
    // default plugin. The unwizened version should fail because the
    // underlying Javy runtime has not been initialized yet. Using `init-plugin` on
    // the unwizened plugin should initialize the runtime so calling
    // `compile-src` on this module should succeed.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasi_common::sync::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().build();
    let mut store = Store::new(&engine, wasi);

    let uninitialized_plugin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(
            std::path::Path::new("target")
                .join("wasm32-wasip1")
                .join("release")
                .join("plugin.wasm"),
        );

    // Check that plugin is in fact uninitialized at this point.
    let module = Module::from_file(&engine, &uninitialized_plugin)?;
    let instance = linker.instantiate(store.as_context_mut(), &module)?;
    let result = instance
        .get_typed_func::<(i32, i32), i32>(store.as_context_mut(), "compile_src")?
        .call(store.as_context_mut(), (0, 0));
    // This should fail because the runtime is uninitialized.
    assert!(result.is_err());

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
        .get_typed_func::<(i32, i32), i32>(store.as_context_mut(), "compile_src")?
        .call(store.as_context_mut(), (0, 0))?;
    Ok(())
}

fn run_with_u8s(r: &mut Runner, stdin: u8) -> (u8, String, u64) {
    let (output, logs, fuel_consumed) = run(r, &stdin.to_le_bytes());
    assert_eq!(1, output.len());
    (output[0], logs, fuel_consumed)
}

fn run(r: &mut Runner, stdin: &[u8]) -> (Vec<u8>, String, u64) {
    run_fn(r, "_start", stdin)
}

fn run_fn(r: &mut Runner, func: &str, stdin: &[u8]) -> (Vec<u8>, String, u64) {
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
    let threshold = 2.0;
    let percentage_difference = ((fuel_consumed - target_fuel) / target_fuel).abs() * 100.0;

    assert!(
        percentage_difference <= threshold,
        "fuel_consumed ({}) was not within {:.2}% of the target_fuel value ({})",
        fuel_consumed,
        threshold,
        target_fuel
    );
}
