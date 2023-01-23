mod runner;

use runner::Runner;

#[test]
fn test_identity() {
    let mut runner = Runner::default();

    let (output, _) = run_with_u8s(&mut runner, 42);
    assert_eq!(42, output);
}

#[test]
fn test_fib() {
    let mut runner = Runner::new("fib.js");

    let (output, _) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
}

#[test]
fn test_recursive_fib() {
    let mut runner = Runner::new("recursive-fib.js");

    let (output, _) = run_with_u8s(&mut runner, 5);
    assert_eq!(8, output);
}

#[test]
fn test_str() {
    let mut runner = Runner::new("str.js");

    let (output, _) = run(&mut runner, "hello".as_bytes());
    assert_eq!("world".as_bytes(), output);
}

#[test]
fn test_encoding() {
    let mut runner = Runner::new("text-encoding.js");

    let (output, _) = run(&mut runner, "hello".as_bytes());
    assert_eq!("el".as_bytes(), output);

    let (output, _) = run(&mut runner, "invalid".as_bytes());
    assert_eq!("true".as_bytes(), output);

    let (output, _) = run(&mut runner, "invalid_fatal".as_bytes());
    assert_eq!("The encoded data was not valid utf-8".as_bytes(), output);

    let (output, _) = run(&mut runner, "test".as_bytes());
    assert_eq!("test2".as_bytes(), output);
}

#[test]
fn test_logging() {
    let mut runner = Runner::new("logging.js");

    let (_output, logs) = run(&mut runner, &[]);
    assert_eq!(
        "hello world from console.log\nhello world from console.error\n",
        logs.as_str(),
    );
}

#[test]
fn test_readme_script() {
    let mut runner = Runner::new("readme.js");

    let (output, _) = run(&mut runner, r#"{ "n": 2, "bar": "baz" }"#.as_bytes());
    assert_eq!(r#"{"foo":3,"newBar":"baz!"}"#.as_bytes(), output);
}

fn run_with_u8s(r: &mut Runner, stdin: u8) -> (u8, String) {
    let (output, logs) = run(r, &stdin.to_le_bytes());
    assert_eq!(1, output.len());
    (output[0], logs)
}

fn run(r: &mut Runner, stdin: &[u8]) -> (Vec<u8>, String) {
    let (output, logs) = r.exec(stdin).unwrap();
    let logs = String::from_utf8(logs).unwrap();
    (output, logs)
}
