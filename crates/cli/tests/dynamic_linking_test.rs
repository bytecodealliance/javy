use anyhow::Result;
use javy_runner::{Builder, Plugin};
use javy_test_macros::javy_cli_test;

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("console.js").build()?;
    assert_wasm_size_within_threshold(388, runner.wasm.len());

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
    assert_wasm_size_within_threshold(615, runner.wasm.len());

    let (_, logs, _) = runner.exec_func("foo-bar", vec![])?;

    assert_eq!("Toplevel\nIn foo\n", String::from_utf8(logs)?);
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_func_without_flag(builder: &mut Builder) -> Result<()> {
    let mut runner = builder.input("linking-with-func-without-flag.js").build()?;
    assert_wasm_size_within_threshold(521, runner.wasm.len());

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
    assert_wasm_size_within_threshold(509, runner.wasm.len());

    let res = runner.exec_func("foo", vec![]);

    assert!(
        res.err()
            .unwrap()
            .to_string()
            .contains("error while executing")
    );
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
pub fn test_dynamic_linking_with_arrow_fn(builder: &mut Builder) -> Result<()> {
    let mut runner = builder
        .input("linking-arrow-func.js")
        .wit("linking-arrow-func.wit")
        .world("exported-arrow")
        .build()?;
    assert_wasm_size_within_threshold(529, runner.wasm.len());

    let (_, logs, _) = runner.exec_func("default", vec![])?;

    assert_eq!("42\n", String::from_utf8(logs)?);
    Ok(())
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
fn test_producers_section_present(builder: &mut Builder) -> Result<()> {
    let runner = builder.input("console.js").build()?;
    assert_wasm_size_within_threshold(388, runner.wasm.len());
    runner.assert_producers()
}

#[javy_cli_test(dyn = true, root = "tests/dynamic-linking-scripts")]
fn test_using_runtime_flag_with_dynamic_triggers_error(builder: &mut Builder) -> Result<()> {
    let build_result = builder.input("console.js").text_encoding(false).build();
    assert!(build_result.is_err_and(|e| {
        e.to_string()
            .contains("error: JavaScript runtime options (-J) are not supported when using a plugin (-C plugin=...)")
    }));
    Ok(())
}

#[javy_cli_test(dyn = true)]
fn test_using_wasip1_plugin_with_dynamic_works(builder: &mut Builder) -> Result<()> {
    let plugin = Plugin::UserWasiP1;
    let mut runner = builder
        .plugin(plugin)
        .preload(plugin.namespace().into(), plugin.path())
        .input("plugin.js")
        .build()?;
    assert_wasm_size_within_threshold(463, runner.wasm.len());

    let result = runner.exec(vec![]);
    assert!(result.is_ok(), "Expected ok but got {result:?}");

    Ok(())
}

#[javy_cli_test(dyn = true)]
fn test_using_wasip1_plugin_with_export_works(builder: &mut Builder) -> Result<()> {
    let plugin = Plugin::UserWasiP1;
    let mut runner = builder
        .plugin(plugin)
        .preload(plugin.namespace().into(), plugin.path())
        .input("plugin-exports.js")
        .wit("plugin-exports.wit")
        .world("plugin")
        .build()?;
    assert_wasm_size_within_threshold(597, runner.wasm.len());

    let result = runner.exec_func("fn", vec![]);
    assert!(result.is_ok(), "Expected ok but got {result:?}");

    Ok(())
}

#[javy_cli_test(dyn = true)]
fn test_using_wasip2_plugin_with_dynamic_works(builder: &mut Builder) -> Result<()> {
    let plugin = Plugin::UserWasiP2;
    let mut runner = builder
        .plugin(plugin)
        .preload(plugin.namespace().into(), plugin.path())
        .input("plugin.js")
        .build()?;
    assert_wasm_size_within_threshold(463, runner.wasm.len());

    let result = runner.exec(vec![]);
    assert!(result.is_ok());

    Ok(())
}

#[javy_cli_test(dyn = true)]
fn test_using_wasip2_plugin_with_export_works(builder: &mut Builder) -> Result<()> {
    let plugin = Plugin::UserWasiP2;
    let mut runner = builder
        .plugin(plugin)
        .preload(plugin.namespace().into(), plugin.path())
        .input("plugin-exports.js")
        .wit("plugin-exports.wit")
        .world("plugin")
        .build()?;
    assert_wasm_size_within_threshold(597, runner.wasm.len());

    let result = runner.exec_func("fn", vec![]);
    assert!(result.is_ok(), "Expected ok but got {result:?}");

    Ok(())
}

fn assert_wasm_size_within_threshold(target_size: usize, wasm_size: usize) {
    let target_size = target_size as f64;
    let wasm_size = wasm_size as f64;
    let threshold = 3.0;
    let percentage_difference = ((wasm_size - target_size) / target_size).abs() * 100.0;

    assert!(
        percentage_difference <= threshold,
        "wasm_size ({wasm_size}) was not within {threshold:.2}% of the target_size value ({target_size})",
    );
}
