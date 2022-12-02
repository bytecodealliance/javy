mod runner;

use lazy_static::lazy_static;
use runner::Runner;
use std::sync::Mutex;

lazy_static! {
    // We avoid running the tests concurrently since the CLI writes on disk at very specific
    // locations, which causes the tests to be unpredictable.
    static ref EXCLUSIVE_TEST: Mutex<()> = Mutex::default();
}

#[test]
fn test_identity() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::default();

    let (output, _) = run_with_u8s(&mut runner, 42);
    assert_eq!(42, output);
}

#[test]
fn test_fib() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("fib.js");

    let (output, _) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
}

#[test]
fn test_recursive_fib() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("recursive-fib.js");

    let (output, _) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
}

#[test]
fn test_str() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("str.js");

    let (output, _) = run(&mut runner, "hello".as_bytes());
    assert_eq!("world".as_bytes(), output);
}

#[test]
fn test_encoding() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("text-encoding.js");

    let (output, _) = run(&mut runner, "hello".as_bytes());
    assert_eq!("el".as_bytes(), output);

    let (output, _) = run(&mut runner, "invalid".as_bytes());
    assert_eq!("true".as_bytes(), output);

    let (output, _) = run(&mut runner, "invalid_fatal".as_bytes());
    assert_eq!("The encoded data was not valid".as_bytes(), output);

    let (output, _) = run(&mut runner, "test".as_bytes());
    assert_eq!("test2".as_bytes(), output);
}

#[test]
fn test_logging() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("logging.js");

    let (output, logs) = run_with_u8s(&mut runner, 42);
    assert_eq!(42, output);
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
}

fn run_with_u8s(r: &mut Runner, i: u8) -> (u8, String) {
    let (output, logs) = run(r, &i.to_le_bytes());
    assert_eq!(1, output.len());
    (output[0], logs)
}

fn run(r: &mut Runner, i: &[u8]) -> (Vec<u8>, String) {
    let (output, logs) = r.exec(i).unwrap();
    let logs = String::from_utf8(logs).unwrap();
    (output, logs)
}
