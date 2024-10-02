use anyhow::Result;
use javy_runner::{Runner, RunnerError, UseExportedFn};
use std::path::{Path, PathBuf};
use std::str;

static ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn test_dylib() -> Result<()> {
    let js_src = "console.log(42);";
    let mut runner = Runner::with_dylib(provider_module()?)?;

    let (_, logs, _) = runner.exec_through_dylib(js_src, UseExportedFn::EvalBytecode)?;
    assert_eq!("42\n", str::from_utf8(&logs)?);

    Ok(())
}

#[test]
fn test_dylib_with_invoke_with_no_fn_name() -> Result<()> {
    let js_src = "console.log(42);";
    let mut runner = Runner::with_dylib(provider_module()?)?;

    let (_, logs, _) = runner.exec_through_dylib(js_src, UseExportedFn::Invoke)?;
    assert_eq!("42\n", str::from_utf8(&logs)?);

    Ok(())
}

#[test]
fn test_dylib_with_error() -> Result<()> {
    let js_src = "function foo() { throw new Error('foo error'); } foo();";

    let mut runner = Runner::with_dylib(provider_module()?)?;

    let res = runner.exec_through_dylib(js_src, UseExportedFn::EvalBytecode);

    assert!(res.is_err());

    let e = res.err().unwrap();
    let expected_log_output = "Error:1:24 foo error\n    at foo (function.mjs:1:24)\n    at <anonymous> (function.mjs:1:50)\n\n";
    assert_eq!(
        expected_log_output,
        String::from_utf8(e.downcast_ref::<RunnerError>().unwrap().stderr.clone())?
    );

    Ok(())
}

#[test]
fn test_dylib_with_exported_func() -> Result<()> {
    let js_src = "export function foo() { console.log('In foo'); }; console.log('Toplevel');";

    let mut runner = Runner::with_dylib(provider_module()?)?;

    let (_, logs, _) = runner.exec_through_dylib(js_src, UseExportedFn::InvokeWithFn("foo"))?;
    assert_eq!("Toplevel\nIn foo\n", str::from_utf8(&logs)?);

    Ok(())
}

fn provider_module() -> Result<Vec<u8>> {
    let mut lib_path = PathBuf::from(ROOT);
    lib_path.pop();
    lib_path.pop();
    lib_path = lib_path.join(
        Path::new("target")
            .join("wasm32-wasi")
            .join("release")
            .join("javy_quickjs_provider_wizened.wasm"),
    );

    std::fs::read(lib_path).map_err(Into::into)
}
