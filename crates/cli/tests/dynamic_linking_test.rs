use anyhow::Result;
use javy_runner::Builder;
use std::path::{Path, PathBuf};
use std::str;

mod common;
use common::run_with_compile_and_build;

static ROOT: &str = env!("CARGO_MANIFEST_DIR");
static BIN: &str = env!("CARGO_BIN_EXE_javy");

#[test]
pub fn test_dynamic_linking() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(root())
            .bin(BIN)
            .input("console.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .build()?;

        let (_, logs, _) = runner.exec(&[])?;
        assert_eq!("42\n", String::from_utf8(logs)?);
        Ok(())
    })
}

#[test]
pub fn test_dynamic_linking_with_func() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(root())
            .bin(BIN)
            .input("linking-with-func.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .wit("linking-with-func.wit")
            .world("foo-test")
            .build()?;

        let (_, logs, _) = runner.exec_func("foo-bar", &[])?;

        assert_eq!("Toplevel\nIn foo\n", String::from_utf8(logs)?);
        Ok(())
    })
}

#[test]
pub fn test_dynamic_linking_with_func_without_flag() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(root())
            .bin(BIN)
            .input("linking-with-func-without-flag.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
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
fn test_errors_in_exported_functions_are_correctly_reported() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(root())
            .bin(BIN)
            .input("errors-in-exported-functions.js")
            .wit("errors-in-exported-functions.wit")
            .world("foo-test")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .build()?;

        let res = runner.exec_func("foo", &[]);

        assert!(res
            .err()
            .unwrap()
            .to_string()
            .contains("error while executing"));
        Ok(())
    })
}

#[test]
// If you need to change this test, then you've likely made a breaking change.
pub fn check_for_new_imports() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let runner = builder
            .root(root())
            .bin(BIN)
            .input("console.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .build()?;

        runner.assert_known_base_imports()
    })
}

#[test]
// If you need to change this test, then you've likely made a breaking change.
pub fn check_for_new_imports_for_exports() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let runner = builder
            .root(root())
            .bin(BIN)
            .input("linking-with-func.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .wit("linking-with-func.wit")
            .world("foo-test")
            .build()?;

        runner.assert_known_named_function_imports()
    })
}

#[test]
pub fn test_dynamic_linking_with_arrow_fn() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let mut runner = builder
            .root(root())
            .bin(BIN)
            .input("linking-arrow-func.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .wit("linking-arrow-func.wit")
            .world("exported-arrow")
            .build()?;

        let (_, logs, _) = runner.exec_func("default", &[])?;

        assert_eq!("42\n", String::from_utf8(logs)?);
        Ok(())
    })
}

#[test]
fn test_producers_section_present() -> Result<()> {
    run_with_compile_and_build(|builder| {
        let runner = builder
            .root(root())
            .bin(BIN)
            .input("console.js")
            .preload("javy_quickjs_provider_v2".into(), provider_module_path())
            .build()?;

        runner.assert_producers()
    })
}

#[test]
// Temporarily ignore given that Javy.JSON is disabled by default.
#[ignore]
fn javy_json_identity() -> Result<()> {
    let mut runner = Builder::default()
        .root(root())
        .bin(BIN)
        .input("javy-json-id.js")
        .preload("javy_quickjs_provider_v2".into(), provider_module_path())
        .build()?;

    let input = "{\"x\":5}";

    let bytes = String::from(input).into_bytes();
    let (out, logs, _) = runner.exec(&bytes)?;

    assert_eq!(String::from_utf8(out)?, "undefined\n");
    assert_eq!(String::from(input), String::from_utf8(logs)?);

    Ok(())
}

fn provider_module_path() -> PathBuf {
    let mut lib_path = PathBuf::from(ROOT);
    lib_path.pop();
    lib_path.pop();
    lib_path = lib_path.join(
        Path::new("target")
            .join("wasm32-wasi")
            .join("release")
            .join("javy_quickjs_provider_wizened.wasm"),
    );

    lib_path
}

fn root() -> PathBuf {
    PathBuf::from(ROOT)
        .join("tests")
        .join("dynamic-linking-scripts")
}
