mod runner;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    // We avoid running the tests concurrently since the CLI writes on disk at very specific
    // locations, which causes the tests to be unpredictable.
    static ref EXCLUSIVE_TEST: Mutex<()> = Mutex::default();
}

#[test]
fn test_identity() {
    let _guard = EXCLUSIVE_TEST.lock();

    let mut runner = runner::Runner::default();

    let input = rmp_serde::to_vec(&42).unwrap();
    let output = runner.exec(input).unwrap();
    let output = rmp_serde::from_slice::<u32>(&output).unwrap();

    assert_eq!(42, output);
}

#[test]
fn test_fib() {
    let _guard = EXCLUSIVE_TEST.lock();

    let mut runner = runner::Runner::new("fib.js");

    let input = rmp_serde::to_vec(&5).unwrap();
    let output = runner.exec(input).unwrap();
    let output = rmp_serde::from_slice::<u32>(&output).unwrap();

    assert_eq!(8, output);
}

#[test]
fn test_recursive_fib() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = runner::Runner::new("recursive-fib.js");

    let input = rmp_serde::to_vec(&5).unwrap();
    let output = runner.exec(input).unwrap();
    let output = rmp_serde::from_slice::<u32>(&output).unwrap();

    assert_eq!(8, output);
}
