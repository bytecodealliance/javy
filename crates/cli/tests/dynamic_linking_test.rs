use anyhow::Result;
use javy_runner::{Builder, Plugin};
use javy_test_macros::javy_cli_test;

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("console.js").build()?;

    let (_, logs, _) = runner.exec(vec![])?;
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

    let (_, logs, _) = runner.exec_func("foo-bar", vec![])?;

    assert_eq!("Toplevel\nIn foo\n", String::from_utf8(logs)?);
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_func_without_flag(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("linking-with-func-without-flag.js").build()?;

    let res = runner.exec_func("foo", vec![]);

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

    let res = runner.exec_func("foo", vec![]);

    assert!(res
        .err()
        .unwrap()
        .to_string()
        .contains("error while executing"));
    Ok(())
}

#[javy_cli_test(
    dyn = true,
    root = "tests/dynamic-linking-scripts",
    commands(not(Compile))
)]
// If you need to change this test, then you've likely made a breaking change.
pub fn check_for_new_imports(builder: &mut Builder) -> Result<()> {
    let runner = builder.input("console.js").build()?;
    runner.ensure_expected_imports(false)
}

#[javy_cli_test(
    dyn = true,
    root = "tests/dynamic-linking-scripts",
    commands(not(Build))
)]
// If you need to change this test, then you've likely made a breaking change.
pub fn check_for_new_imports_for_compile(builder: &mut Builder) -> Result<()> {
    let runner = builder.input("console.js").build()?;
    runner.ensure_expected_imports(true)
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_arrow_fn(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("linking-arrow-func.js")
        .wit("linking-arrow-func.wit")
        .world("exported-arrow")
        .build()?;

    let (_, logs, _) = runner.exec_func("default", vec![])?;

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
    let build_result = builder.input("console.js").text_encoding(false).build();
    assert!(build_result.is_err_and(|e| e
        .to_string()
        .contains("error: Property text-encoding is not supported for runtime configuration")));
    Ok(())
}

#[javy_cli_test(dyn = true, commands(not(Compile)))]
fn test_using_plugin_with_dynamic_works(builder: &mut Builder) -> Result<()> {
    let plugin = Plugin::User;
    let mut runner = builder
        .plugin(Plugin::User)
        .preload(plugin.namespace().into(), plugin.path())
        .input("plugin.js")
        .build()?;

    let result = runner.exec(vec![]);
    assert!(result.is_ok());

    Ok(())
}
