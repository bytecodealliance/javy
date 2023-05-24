mod runner;

use runner::Runner;
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

    let (output, _, fuel_consumed) = run(&mut runner, &[]);
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
fn test_producers_section_present() {
    let runner = Runner::new("readme.js");
    let producers_section = wasmparser::Parser::new(0)
        .parse_all(&runner.wasm)
        .find_map(|payload| match payload {
            Ok(wasmparser::Payload::CustomSection(c)) if c.name() == "producers" => Some(c.data()),
            _ => None,
        })
        .unwrap();
    let producers_string = str::from_utf8(producers_section).unwrap();
    assert!(producers_string.contains("JavaScript"));
    assert!(producers_string.contains("Javy"));
}

#[test]
fn test_same_module_outputs_different_random_result() {
    let mut runner = Runner::new("random.js");
    let (output, _, fuel_consumed) = runner.exec(&[]).unwrap();
    let (output2, _, _) = runner.exec(&[]).unwrap();
    // in theory these could be equal with a correct implementation but it's very unlikely.
    assert!(output != output2);
    assert_fuel_consumed_within_threshold(100_543, fuel_consumed);
}

fn run_with_u8s(r: &mut Runner, stdin: u8) -> (u8, String, u64) {
    let (output, logs, fuel_consumed) = run(r, &stdin.to_le_bytes());
    assert_eq!(1, output.len());
    (output[0], logs, fuel_consumed)
}

fn run(r: &mut Runner, stdin: &[u8]) -> (Vec<u8>, String, u64) {
    let (output, logs, fuel_consumed) = r.exec(stdin).unwrap();
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
