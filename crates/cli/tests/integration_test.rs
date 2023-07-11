mod common;
mod runner;

use runner::{Runner, RunnerError};
use std::str;

#[test]
fn test_identity() {
    let mut runner = Runner::default();

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 42);
    assert_eq!(42, output);
    assert_fuel_consumed_within_threshold(37907, fuel_consumed);
}

#[test]
fn test_fib() {
    let mut runner = Runner::new("fib.js");

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
    assert_fuel_consumed_within_threshold(59_822, fuel_consumed);
}

#[test]
fn test_recursive_fib() {
    let mut runner = Runner::new("recursive-fib.js");

    let (output, _, fuel_consumed) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
    assert_fuel_consumed_within_threshold(61_617, fuel_consumed);
}

#[test]
fn test_str() {
    let mut runner = Runner::new("str.js");

    let (output, _, fuel_consumed) = run(&mut runner, "hello".as_bytes());
    assert_eq!("world".as_bytes(), output);
    assert_fuel_consumed_within_threshold(142_849, fuel_consumed);
}

#[test]
fn test_encoding() {
    let mut runner = Runner::new("text-encoding.js");

    let (output, _, fuel_consumed) = run(&mut runner, "hello".as_bytes());
    assert_eq!("el".as_bytes(), output);
    assert_fuel_consumed_within_threshold(258_852, fuel_consumed);

    let (output, _, _) = run(&mut runner, "invalid".as_bytes());
    assert_eq!("true".as_bytes(), output);

    let (output, _, _) = run(&mut runner, "invalid_fatal".as_bytes());
    assert_eq!("The encoded data was not valid utf-8".as_bytes(), output);

    let (output, _, _) = run(&mut runner, "test".as_bytes());
    assert_eq!("test2".as_bytes(), output);
}

#[test]
fn test_logging() {
    let mut runner = Runner::new("logging.js");

    let (_output, logs, fuel_consumed) = run(&mut runner, &[]);
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
    assert_fuel_consumed_within_threshold(22_296, fuel_consumed);
}

#[test]
fn test_readme_script() {
    let mut runner = Runner::new("readme.js");

    let (output, _, fuel_consumed) = run(&mut runner, r#"{ "n": 2, "bar": "baz" }"#.as_bytes());
    assert_eq!(r#"{"foo":3,"newBar":"baz!"}"#.as_bytes(), output);
    assert_fuel_consumed_within_threshold(284_736, fuel_consumed);
}

#[cfg(feature = "experimental_event_loop")]
#[test]
fn test_promises() {
    let mut runner = Runner::new("promise.js");

    let (output, _, _) = run(&mut runner, &[]);
    assert_eq!("\"foo\"\"bar\"".as_bytes(), output);
}

#[cfg(not(feature = "experimental_event_loop"))]
#[test]
fn test_promises() {
    use crate::runner::RunnerError;

    let mut runner = Runner::new("promise.js");
    let res = runner.exec(&[]);
    let err = res.err().unwrap().downcast::<RunnerError>().unwrap();
    assert!(str::from_utf8(&err.stderr)
        .unwrap()
        .contains("Adding tasks to the event queue is not supported"));
}

#[test]
fn test_exported_functions() {
    let mut runner = Runner::new_with_exports("exported-fn.js", "exported-fn.wit", "exported-fn");
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "foo", &[]);
    assert_eq!("Hello from top-level\nHello from foo\n", logs);
    assert_fuel_consumed_within_threshold(54610, fuel_consumed);
    let (_, logs, _) = run_fn(&mut runner, "foo-bar", &[]);
    assert_eq!("Hello from top-level\nHello from fooBar\n", logs);
}

#[cfg(feature = "experimental_event_loop")]
#[test]
fn test_exported_promises() {
    let mut runner = Runner::new_with_exports(
        "exported-promise-fn.js",
        "exported-promise-fn.wit",
        "exported-promise-fn",
    );
    let (_, logs, _) = run_fn(&mut runner, "foo", &[]);
    assert_eq!("Top-level\ninside foo\n", logs);
}

#[test]
fn test_exported_functions_without_flag() {
    let mut runner = Runner::new("exported-fn.js");
    let res = runner.exec_func("foo", &[]);
    assert_eq!(
        "failed to find function export `foo`",
        res.err().unwrap().to_string()
    );
}

#[test]
fn test_producers_section_present() {
    let runner = Runner::new("readme.js");
    common::assert_producers_section_is_correct(&runner.wasm).unwrap();
}

#[test]
fn test_error_handling() {
    let mut runner = Runner::new("error.js");
    let result = runner.exec(&[]);
    let err = result.err().unwrap().downcast::<RunnerError>().unwrap();

    let expected_log_output = "Error while running JS: Uncaught Error: error\n    at error (function.mjs:2)\n    at <anonymous> (function.mjs:5)\n\n";

    assert_eq!(expected_log_output, str::from_utf8(&err.stderr).unwrap());
}

#[test]
fn test_same_module_outputs_different_random_result() {
    let mut runner = Runner::new("random.js");
    let (output, _, _) = runner.exec(&[]).unwrap();
    let (output2, _, _) = runner.exec(&[]).unwrap();
    // In theory these could be equal with a correct implementation but it's very unlikely.
    assert!(output != output2);
    // Don't check fuel consumed because fuel consumed can be different from run to run. See
    // https://github.com/bytecodealliance/javy/issues/401 for investigating the cause.
}

#[test]
fn test_exported_default_arrow_fn() {
    let mut runner = Runner::new_with_exports(
        "exported-default-arrow-fn.js",
        "exported-default-arrow-fn.wit",
        "exported-arrow",
    );
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", &[]);
    assert_eq!(logs, "42\n");
    assert_fuel_consumed_within_threshold(48_628, fuel_consumed);
}

#[test]
fn test_exported_default_fn() {
    let mut runner = Runner::new_with_exports(
        "exported-default-fn.js",
        "exported-default-fn.wit",
        "exported-default",
    );
    let (_, logs, fuel_consumed) = run_fn(&mut runner, "default", &[]);
    assert_eq!(logs, "42\n");
    assert_fuel_consumed_within_threshold(49_748, fuel_consumed);
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

/// Used to detect any significant changes in the fuel consumption when making changes in Javy.
///
/// A threshold is used here so that we can decide how much of a change is acceptable. The threshold value needs to be sufficiently large enough to account for fuel differences between different operating systems.
///
/// If the fuel_consumed is less than target_fuel, then great job decreasing the fuel consumption!
/// However, if the fuel_consumed is greater than target_fuel and over the threshold, please consider if the changes are worth the increase in fuel consumption.
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
