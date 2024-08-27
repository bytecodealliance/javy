use anyhow::Result;
use javy_runner::{Builder, Runner, RunnerError};
use std::path::PathBuf;
use std::str;

mod common;
use common::run_with_compile_and_build;

static BIN: &str = env!("CARGO_BIN_EXE_javy");
static ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn test_identity() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder.root(sample_scripts()).bin(BIN).build()?;

        let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 42);
        assert_eq!(42, output);
        assert_fuel_consumed_within_threshold(47_773, fuel_consumed);
        Ok(())
    })
}

#[test]
fn test_fib() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("fib.js")
            .build()?;

        let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 5);
        assert_eq!(8, output);
        assert_fuel_consumed_within_threshold(66_007, fuel_consumed);
        Ok(())
    })
}

#[test]
fn test_recursive_fib() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("recursive-fib.js")
            .build()?;

        let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 5);
        assert_eq!(8, output);
        assert_fuel_consumed_within_threshold(69_306, fuel_consumed);
        Ok(())
    })
}

#[test]
fn test_str() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("str.js")
            .build()?;

        let (output, _, fuel_consumed) = run(&mut runner, "hello".as_bytes());
        assert_eq!("world".as_bytes(), output);
        assert_fuel_consumed_within_threshold(142_849, fuel_consumed);
        Ok(())
    })
}

#[test]
fn test_encoding() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("text-encoding.js")
            .build()?;

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
    })
}

#[test]
fn test_logging_with_compile() -> Result<()> {
    let mut runner = Builder::default()
        .root(sample_scripts())
        .bin(BIN)
        .input("logging.js")
        .command(javy_runner::JavyCommand::Compile)
        .build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert!(output.is_empty());
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
    assert_fuel_consumed_within_threshold(34169, fuel_consumed);
    Ok(())
}

#[test]
fn test_logging_without_redirect() -> Result<()> {
    let mut runner = Builder::default()
        .root(sample_scripts())
        .bin(BIN)
        .input("logging.js")
        .command(javy_runner::JavyCommand::Build)
        .redirect_stdout_to_stderr(false)
        .build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert_eq!(b"hello world from console.log\n".to_vec(), output);
    assert_eq!("hello world from console.error\n", logs.as_str());
    assert_fuel_consumed_within_threshold(34169, fuel_consumed);
    Ok(())
}

#[test]
fn test_logging_with_redirect() -> Result<()> {
    let mut runner = Builder::default()
        .root(sample_scripts())
        .bin(BIN)
        .input("logging.js")
        .command(javy_runner::JavyCommand::Build)
        .redirect_stdout_to_stderr(true)
        .build()?;

    let (output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert!(output.is_empty());
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
    assert_fuel_consumed_within_threshold(34169, fuel_consumed);
    Ok(())
}

#[test]
fn test_readme_script() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("readme.js")
            .build()?;

        let (output, _, fuel_consumed) = run(&mut runner, r#"{ "n": 2, "bar": "baz" }"#.as_bytes());
        assert_eq!(r#"{"foo":3,"newBar":"baz!"}"#.as_bytes(), output);
        assert_fuel_consumed_within_threshold(270_919, fuel_consumed);
        Ok(())
    })
}

#[cfg(feature = "experimental_event_loop")]
#[test]
fn test_promises() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("promise.js")
            .build()?;

        let (output, _, _) = run(&mut runner, &[]);
        assert_eq!("\"foo\"\"bar\"".as_bytes(), output);
        Ok(())
    })
}

#[cfg(not(feature = "experimental_event_loop"))]
#[test]
fn test_promises() -> Result<()> {
    use javy_runner::RunnerError;

    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("promise.js")
            .build()?;
        let res = runner.exec(&[]);
        let err = res.err().unwrap().downcast::<RunnerError>().unwrap();
        assert!(str::from_utf8(&err.stderr)
            .unwrap()
            .contains("Pending jobs in the event queue."));

        Ok(())
    })
}

#[cfg(feature = "experimental_event_loop")]
#[test]
fn test_promise_top_level_await() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("top-level-await.js")
            .build()?;
        let (out, _, _) = run(&mut runner, &[]);

        assert_eq!("bar", String::from_utf8(out)?);
        Ok(())
    })
}

#[test]
fn test_exported_functions() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("exported-fn.js")
            .wit("exported-fn.wit")
            .world("exported-fn")
            .build()?;
        let (_, logs, fuel_consumed) = run_fn(&mut runner, "foo", &[]);
        assert_eq!("Hello from top-level\nHello from foo\n", logs);
        assert_fuel_consumed_within_threshold(80023, fuel_consumed);
        let (_, logs, _) = run_fn(&mut runner, "foo-bar", &[]);
        assert_eq!("Hello from top-level\nHello from fooBar\n", logs);
        Ok(())
    })
}

#[cfg(feature = "experimental_event_loop")]
#[test]
fn test_exported_promises() -> Result<()> {
    use clap::builder;

    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("exported-promise-fn.js")
            .wit("exported-promise-fn.wit")
            .world("exported-promise-fn")
            .build()?;
        let (_, logs, _) = run_fn(&mut runner, "foo", &[]);
        assert_eq!("Top-level\ninside foo\n", logs);
        Ok(())
    })
}

#[test]
fn test_exported_functions_without_flag() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("exported-fn.js")
            .build()?;
        let res = runner.exec_func("foo", &[]);
        assert_eq!(
            "failed to find function export `foo`",
            res.err().unwrap().to_string()
        );
        Ok(())
    })
}

#[test]
fn test_exported_function_without_semicolons() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("exported-fn-no-semicolon.js")
            .wit("exported-fn-no-semicolon.wit")
            .world("exported-fn")
            .build()?;
        run_fn(&mut runner, "foo", &[]);
        Ok(())
    })
}

#[test]
fn test_producers_section_present() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("readme.js")
            .build()?;

        runner.assert_producers()
    })
}

#[test]
fn test_error_handling() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(sample_scripts())
            .bin(BIN)
            .input("error.js")
            .build()?;
        let result = runner.exec(&[]);
        let err = result.err().unwrap().downcast::<RunnerError>().unwrap();

        let expected_log_output = "Error:2:9 error\n    at error (function.mjs:2:9)\n    at <anonymous> (function.mjs:5:1)\n\n";

        assert_eq!(expected_log_output, str::from_utf8(&err.stderr).unwrap());
        Ok(())
    })
}

#[test]
fn test_same_module_outputs_different_random_result() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("random.js")
            .build()?;
        let (output, _, _) = runner.exec(&[]).unwrap();
        let (output2, _, _) = runner.exec(&[]).unwrap();
        // In theory these could be equal with a correct implementation but it's very unlikely.
        assert!(output != output2);
        // Don't check fuel consumed because fuel consumed can be different from run to run. See
        // https://github.com/bytecodealliance/javy/issues/401 for investigating the cause.
        Ok(())
    })
}

#[test]
fn test_exported_default_arrow_fn() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("exported-default-arrow-fn.js")
            .wit("exported-default-arrow-fn.wit")
            .world("exported-arrow")
            .build()?;

        let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", &[]);
        assert_eq!(logs, "42\n");
        assert_fuel_consumed_within_threshold(76706, fuel_consumed);
        Ok(())
    })
}

#[test]
fn test_exported_default_fn() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .bin(BIN)
            .root(sample_scripts())
            .input("exported-default-fn.js")
            .wit("exported-default-fn.wit")
            .world("exported-default")
            .build()?;
        let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", &[]);
        assert_eq!(logs, "42\n");
        assert_fuel_consumed_within_threshold(77909, fuel_consumed);
        Ok(())
    })
}

fn sample_scripts() -> PathBuf {
    PathBuf::from(ROOT).join("tests").join("sample-scripts")
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
    let threshold = 10.0;
    let percentage_difference = ((fuel_consumed - target_fuel) / target_fuel).abs() * 100.0;

    assert!(
        percentage_difference <= threshold,
        "fuel_consumed ({}) was not within {:.2}% of the target_fuel value ({})",
        fuel_consumed,
        threshold,
        target_fuel
    );
}
