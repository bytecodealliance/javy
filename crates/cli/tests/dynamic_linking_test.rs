use anyhow::Result;
use javy_runner::{Builder, Plugin};
use javy_test_macros::javy_cli_test;

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("console.js").build()?;

    let (_, logs, _) = runner.exec(&[])?;
    assert_eq!("42\n", String::from_utf8(logs)?);
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_func(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("linking-with-func.js")
        .wit("linking-with-func.wit")
        .world("foo-test")
        .build()?;

    let (_, logs, _) = runner.exec_func("foo-bar", &[])?;

    assert_eq!("Toplevel\nIn foo\n", String::from_utf8(logs)?);
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_func_without_flag(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("linking-with-func-without-flag.js").build()?;

    let res = runner.exec_func("foo", &[]);

    assert_eq!(
        "failed to find function export `foo`",
        res.err().unwrap().to_string()
    );
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
fn test_errors_in_exported_functions_are_correctly_reported(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("errors-in-exported-functions.js")
        .wit("errors-in-exported-functions.wit")
        .world("foo-test")
        .build()?;

    let res = runner.exec_func("foo", &[]);

    assert!(res
        .err()
        .unwrap()
        .to_string()
        .contains("error while executing"));
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
// If you need to change this test, then you've likely made a breaking change.
pub fn check_for_new_imports(builder: &mut Builder) -> Result<()> {
    let runner = builder.input("console.js").build()?;
    runner.assert_known_base_imports()
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
// If you need to change this test, then you've likely made a breaking change.
pub fn check_for_new_imports_for_exports(builder: &mut Builder) -> Result<()> {
    let runner = builder
        .input("linking-with-func.js")
        .wit("linking-with-func.wit")
        .world("foo-test")
        .build()?;

    runner.assert_known_named_function_imports()
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_arrow_fn(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("linking-arrow-func.js")
        .wit("linking-arrow-func.wit")
        .world("exported-arrow")
        .build()?;

    let (_, logs, _) = runner.exec_func("default", &[])?;

    assert_eq!("42\n", String::from_utf8(logs)?);
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
fn test_producers_section_present(builder: &mut Builder) -> Result<()> {
    let runner = builder.input("console.js").build()?;
    runner.assert_producers()
}

#[javy_cli_test(
    dyn = true,
    root = "tests/dynamic-linking-scripts",
    commands(not(Compile))
)]
fn test_using_runtime_flag_with_dynamic_triggers_error(builder: &mut Builder) -> Result<()> {
    let build_result = builder
        .input("console.js")
        .redirect_stdout_to_stderr(false)
        .build();
    assert!(build_result.is_err_and(|e| e
        .to_string()
        .contains("Error: Cannot set JS runtime options when building a dynamic module")));
    Ok(())
}

#[javy_cli_test(
    dyn = true,
    root = "tests/dynamic-linking-scripts",
    commands(not(Compile))
)]
fn javy_json_identity(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("javy-json-id.js").build()?;

    let input = "{\"x\":5}";

    let bytes = String::from(input).into_bytes();
    let (out, logs, _) = runner.exec(&bytes)?;

    assert_eq!(String::from_utf8(out)?, input);
    assert_eq!(String::from_utf8(logs)?, "undefined\n");

    Ok(())
}

#[javy_cli_test(dyn = true, commands(not(Compile)))]
fn test_using_plugin_with_dynamic_build_fails(builder: &mut Builder) -> Result<()> {
    let result = builder.plugin(Plugin::User).input("plugin.js").build();
    let err = result.err().unwrap();
    assert!(err
        .to_string()
        .contains("Cannot use plugins for building dynamic modules"));

    Ok(())
}
