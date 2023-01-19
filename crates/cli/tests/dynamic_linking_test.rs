use anyhow::Result;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::process::Command;
use std::str;
use uuid::Uuid;
use wasi_common::pipe::WritePipe;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

mod common;

#[test]
pub fn test_dynamic_linking() -> Result<()> {
    let js_src = "console.log(42);";
    let js_wasm = create_dynamically_linked_wasm_module(js_src)?;

    let stderr = WritePipe::new_in_memory();
    // scope is needed to ensure `store` is dropped before trying to read from `stderr` below
    {
        let (engine, mut linker, mut store) = create_wasm_env(stderr.clone())?;
        let quickjs_provider_module = common::create_quickjs_provider_module(&engine)?;
        let js_module = Module::from_binary(&engine, &js_wasm)?;

        let quickjs_provider_instance = linker.instantiate(&mut store, &quickjs_provider_module)?;
        linker.instance(
            &mut store,
            "javy_quickjs_provider_v1",
            quickjs_provider_instance,
        )?;
        let js_instance = linker.instantiate(&mut store, &js_module)?;
        let start_func = js_instance.get_typed_func::<(), (), _>(&mut store, "_start")?;
        start_func.call(&mut store, ())?;
    }

    let log_output = stderr.try_into_inner().unwrap().into_inner();
    assert_eq!("42\n", str::from_utf8(&log_output)?);

    Ok(())
}

fn create_dynamically_linked_wasm_module(js_src: &str) -> Result<Vec<u8>> {
    let Ok(tempdir) = tempfile::tempdir() else {
        panic!("Could not create temporary directory for .wasm test artifacts");
    };
    let js_path = tempdir.path().join(Uuid::new_v4().to_string());
    let wasm_path = tempdir.path().join(Uuid::new_v4().to_string());

    let mut js_file = File::create(&js_path)?;
    js_file.write_all(js_src.as_bytes())?;
    let output = Command::new(env!("CARGO_BIN_EXE_javy"))
        .arg(&js_path.to_str().unwrap())
        .arg("-o")
        .arg(wasm_path.to_str().unwrap())
        .arg("-d")
        .output()?;
    assert!(output.status.success());

    let mut wasm_file = File::open(&wasm_path)?;
    let mut contents = vec![];
    wasm_file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn create_wasm_env(
    stderr: WritePipe<Cursor<Vec<u8>>>,
) -> Result<(Engine, Linker<WasiCtx>, Store<WasiCtx>)> {
    let engine = Engine::new(Config::new().wasm_multi_memory(true))?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().stderr(Box::new(stderr)).build();
    let store = Store::new(&engine, wasi);
    Ok((engine, linker, store))
}
