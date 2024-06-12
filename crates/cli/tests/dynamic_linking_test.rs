use anyhow::Result;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::process::Command;
use std::str;
use uuid::Uuid;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::sync::WasiCtxBuilder;
use wasi_common::WasiCtx;
use wasmtime::{Config, Engine, ExternType, Linker, Module, Store};

mod common;

#[test]
pub fn test_dynamic_linking() -> Result<()> {
    let js_src = "console.log(42);";
    let log_output = invoke_fn_on_generated_module(js_src, "_start", None, None, None)?;
    assert_eq!("42\n", &log_output);
    Ok(())
}

#[test]
pub fn test_dynamic_linking_with_func() -> Result<()> {
    let js_src = "export function fooBar() { console.log('In foo'); }; console.log('Toplevel');";
    let wit = "
        package local:main

        world foo-test {
            export foo-bar: func()
        }
    ";
    let log_output =
        invoke_fn_on_generated_module(js_src, "foo-bar", Some((wit, "foo-test")), None, None)?;
    assert_eq!("Toplevel\nIn foo\n", &log_output);
    Ok(())
}

#[test]
pub fn test_dynamic_linking_with_func_without_flag() -> Result<()> {
    let js_src = "export function foo() { console.log('In foo'); }; console.log('Toplevel');";
    let res = invoke_fn_on_generated_module(js_src, "foo", None, None, None);
    assert_eq!(
        "failed to find function export `foo`",
        res.err().unwrap().to_string()
    );
    Ok(())
}

#[test]
pub fn check_for_new_imports() -> Result<()> {
    // If you need to change this test, then you've likely made a breaking change.
    let js_src = "console.log(42);";
    let wasm = create_dynamically_linked_wasm_module(js_src, None)?;
    let (engine, _linker, _store) = create_wasm_env(WritePipe::new_in_memory(), None, None)?;
    let module = Module::from_binary(&engine, &wasm)?;
    for import in module.imports() {
        match (import.module(), import.name(), import.ty()) {
            ("javy_quickjs_provider_v2", "canonical_abi_realloc", ExternType::Func(f))
                if f.params().map(|t| t.is_i32()).eq([true, true, true, true])
                    && f.results().map(|t| t.is_i32()).eq([true]) => {}
            ("javy_quickjs_provider_v2", "eval_bytecode", ExternType::Func(f))
                if f.params().map(|t| t.is_i32()).eq([true, true]) && f.results().len() == 0 => {}
            ("javy_quickjs_provider_v2", "memory", ExternType::Memory(_)) => (),
            _ => panic!("Unknown import {:?}", import),
        }
    }
    Ok(())
}

#[test]
pub fn check_for_new_imports_for_exports() -> Result<()> {
    // If you need to change this test, then you've likely made a breaking change.
    let js_src = "export function foo() { console.log('In foo'); }; console.log('Toplevel');";
    let wit = "
        package local:main

        world foo-test {
            export foo: func()
        }
    ";
    let wasm = create_dynamically_linked_wasm_module(js_src, Some((wit, "foo-test")))?;
    let (engine, _linker, _store) = create_wasm_env(WritePipe::new_in_memory(), None, None)?;
    let module = Module::from_binary(&engine, &wasm)?;
    for import in module.imports() {
        match (import.module(), import.name(), import.ty()) {
            ("javy_quickjs_provider_v2", "canonical_abi_realloc", ExternType::Func(f))
                if f.params().map(|t| t.is_i32()).eq([true, true, true, true])
                    && f.results().map(|t| t.is_i32()).eq([true]) => {}
            ("javy_quickjs_provider_v2", "eval_bytecode", ExternType::Func(f))
                if f.params().map(|t| t.is_i32()).eq([true, true]) && f.results().len() == 0 => {}
            ("javy_quickjs_provider_v2", "invoke", ExternType::Func(f))
                if f.params().map(|t| t.is_i32()).eq([true, true, true, true])
                    && f.results().len() == 0 => {}
            ("javy_quickjs_provider_v2", "memory", ExternType::Memory(_)) => (),
            _ => panic!("Unknown import {:?}", import),
        }
    }
    Ok(())
}

#[test]
pub fn test_dynamic_linking_with_arrow_fn() -> Result<()> {
    let js_src = "export default () => console.log(42)";
    let wit = "
        package local:test

        world exported-arrow {
            export default: func()
        }
    ";
    let log_output = invoke_fn_on_generated_module(
        js_src,
        "default",
        Some((wit, "exported-arrow")),
        None,
        None,
    )?;
    assert_eq!("42\n", log_output);
    Ok(())
}

#[test]
fn test_producers_section_present() -> Result<()> {
    let js_wasm = create_dynamically_linked_wasm_module("console.log(42)", None)?;
    common::assert_producers_section_is_correct(&js_wasm)?;
    Ok(())
}

#[test]
fn javy_json_identity() -> Result<()> {
    let src = r#"
        console.log(Javy.JSON.toStdout(Javy.JSON.fromStdin()));
    "#;

    let input = "{\"x\":5}";

    let bytes = String::from(input).into_bytes();
    let stdin = Some(ReadPipe::from(bytes));
    let stdout = WritePipe::new_in_memory();
    let out = invoke_fn_on_generated_module(src, "_start", None, stdin, Some(stdout.clone()))?;

    assert_eq!(out, "undefined\n");
    assert_eq!(
        String::from(input),
        String::from_utf8(stdout.try_into_inner().unwrap().into_inner()).unwrap(),
    );

    Ok(())
}

fn create_dynamically_linked_wasm_module(
    js_src: &str,
    wit: Option<(&str, &str)>,
) -> Result<Vec<u8>> {
    let Ok(tempdir) = tempfile::tempdir() else {
        panic!("Could not create temporary directory for .wasm test artifacts");
    };
    let js_path = tempdir.path().join(Uuid::new_v4().to_string());
    let wit_path = tempdir.path().join(Uuid::new_v4().to_string());
    let wasm_path = tempdir.path().join(Uuid::new_v4().to_string());

    let mut js_file = File::create(&js_path)?;
    js_file.write_all(js_src.as_bytes())?;
    if let Some((wit, _)) = wit {
        fs::write(&wit_path, wit)?;
    }
    let mut args = vec![
        "compile",
        js_path.to_str().unwrap(),
        "-o",
        wasm_path.to_str().unwrap(),
        "-d",
    ];
    if let Some((_, world)) = wit {
        args.push("--wit");
        args.push(wit_path.to_str().unwrap());
        args.push("-n");
        args.push(world);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_javy"))
        .args(args)
        .output()?;
    assert!(output.status.success());

    let mut wasm_file = File::open(&wasm_path)?;
    let mut contents = vec![];
    wasm_file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn invoke_fn_on_generated_module(
    js_src: &str,
    func: &str,
    wit: Option<(&str, &str)>,
    stdin: Option<ReadPipe<Cursor<Vec<u8>>>>,
    stdout: Option<WritePipe<Cursor<Vec<u8>>>>,
) -> Result<String> {
    let js_wasm = create_dynamically_linked_wasm_module(js_src, wit)?;

    let stderr = WritePipe::new_in_memory();
    let (engine, mut linker, mut store) = create_wasm_env(stderr.clone(), stdin, stdout)?;
    let quickjs_provider_module = common::create_quickjs_provider_module(&engine)?;
    let js_module = Module::from_binary(&engine, &js_wasm)?;

    let quickjs_provider_instance = linker.instantiate(&mut store, &quickjs_provider_module)?;
    linker.instance(
        &mut store,
        "javy_quickjs_provider_v2",
        quickjs_provider_instance,
    )?;
    let js_instance = linker.instantiate(&mut store, &js_module)?;
    let func = js_instance.get_typed_func::<(), ()>(&mut store, func)?;
    func.call(&mut store, ())?;

    drop(store); // Need to drop store to access contents of stderr.
    let log_output = stderr.try_into_inner().unwrap().into_inner();

    Ok(String::from_utf8(log_output)?)
}

fn create_wasm_env(
    stderr: WritePipe<Cursor<Vec<u8>>>,
    stdin: Option<ReadPipe<Cursor<Vec<u8>>>>,
    stdout: Option<WritePipe<Cursor<Vec<u8>>>>,
) -> Result<(Engine, Linker<WasiCtx>, Store<WasiCtx>)> {
    let engine = Engine::new(Config::new().wasm_multi_memory(true))?;
    let mut linker = Linker::new(&engine);
    wasi_common::sync::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().stderr(Box::new(stderr)).build();
    if let Some(stdout) = stdout {
        wasi.set_stdout(Box::new(stdout));
    }
    if let Some(stdin) = stdin {
        wasi.set_stdin(Box::new(stdin));
    }
    let store = Store::new(&engine, wasi);
    Ok((engine, linker, store))
}
