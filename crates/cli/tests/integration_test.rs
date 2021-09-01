mod runner;

use runner::Runner;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde::de::DeserializeOwned;
use serde::Serialize;

lazy_static! {
    // We avoid running the tests concurrently since the CLI writes on disk at very specific
    // locations, which causes the tests to be unpredictable.
    static ref EXCLUSIVE_TEST: Mutex<()> = Mutex::default();
}

#[test]
fn test_identity() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::default();

    let output = run::<_, u32>(&mut runner, &42);
    assert_eq!(42, output);
}

#[test]
fn test_fib() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("fib.js");

    let output = run::<_, u32>(&mut runner, &5);
    assert_eq!(8, output);
}

#[test]
fn test_recursive_fib() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("recursive-fib.js");

    let output = run::<_, u32>(&mut runner, &5);
    assert_eq!(8, output);
}

#[test]
fn test_str() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("str.js");

    let output = run::<_, String>(&mut runner, &"hello".to_string());
    assert_eq!("world", output.as_str());
}

#[test]
fn test_big_ints() {
    let _guard = EXCLUSIVE_TEST.lock();
    let mut runner = Runner::new("big-ints.js");

    let output = run::<_, String>(&mut runner, &42);
    assert_eq!("a", output.as_str());

    let output = run::<_, String>(&mut runner, &i64::MAX);
    assert_eq!("b", output.as_str());

    let output = run::<_, String>(&mut runner, &i64::MIN);
    assert_eq!("c", output.as_str());

    let output = run::<_, String>(&mut runner, &u64::MAX);
    assert_eq!("d", output.as_str());

    let output = run::<_, String>(&mut runner, &u64::MIN);
    assert_eq!("e", output.as_str());
}

fn run<I, O>(r: &mut Runner, i: &I) -> O
    where I: Serialize,
          O: DeserializeOwned,
{
    let input = rmp_serde::to_vec(i).unwrap();
    let output = r.exec(input).unwrap();
    let output = rmp_serde::from_slice::<O>(&output).unwrap();
    output
}
