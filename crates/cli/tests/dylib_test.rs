use anyhow::Result;
use std::boxed::Box;
use std::path::{Path, PathBuf};
use std::str;
use wasi_common::pipe::WritePipe;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

mod module_generator;

#[test]
fn test_dylib() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let stderr = WritePipe::new_in_memory();
    let wasi = WasiCtxBuilder::new()
        .stderr(Box::new(stderr.clone()))
        .build();
    let module = create_module(&engine)?;

    // scope is needed to ensure `store` is dropped before trying to read from `stderr` below
    {
        let mut store = Store::new(&engine, wasi);
        let instance = linker.instantiate(&mut store, &module)?;
        let eval_bytecode_func =
            instance.get_typed_func::<(u32, u32), (), _>(&mut store, "eval_bytecode")?;

        let js_src = "console.log(42);";
        let (bytecode_ptr, bytecode_len) = compile_src(js_src.as_bytes(), &instance, &mut store)?;
        eval_bytecode_func.call(&mut store, (bytecode_ptr, bytecode_len))?;
    }

    let log_output = stderr.try_into_inner().unwrap().into_inner();
    assert_eq!("42\n", str::from_utf8(&log_output)?);

    Ok(())
}

#[test]
fn test_dylib_workflow() -> Result<()> {
    let engine = Engine::new(Config::default().wasm_multi_memory(true))?;
    let quickjs_provider_module = create_module(&engine)?;

    let js_src = "console.log(42);";
    let bytecode = compile_src_with_separate_instance(&engine, &quickjs_provider_module, js_src)?;
    let wasm_module = module_generator::generate_module(bytecode, js_src)?;
    let js_module = Module::from_binary(&engine, &wasm_module)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let stderr = WritePipe::new_in_memory();
    let wasi = WasiCtxBuilder::new()
        .stderr(Box::new(stderr.clone()))
        .build();

    // scope is needed to ensure `store` is dropped before trying to read from `stderr` below
    {
        let mut store = Store::new(&engine, wasi);
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

fn create_module(engine: &Engine) -> Result<Module> {
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.pop();
    lib_path.pop();
    lib_path = lib_path.join(
        Path::new("target")
            .join("wasm32-wasi")
            .join("release")
            .join("javy_quickjs_provider.wasm"),
    );
    Module::from_file(engine, lib_path)
}

fn compile_src(
    js_src: &[u8],
    instance: &Instance,
    mut store: &mut Store<WasiCtx>,
) -> Result<(u32, u32)> {
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let compile_src_func =
        instance.get_typed_func::<(u32, u32), u32, _>(&mut store, "compile_src")?;

    let js_src_ptr = allocate_memory(&instance, &mut store, 1, js_src.len().try_into()?)?;
    memory.write(&mut store, js_src_ptr.try_into()?, js_src)?;

    let ret_ptr = compile_src_func.call(&mut store, (js_src_ptr, js_src.len().try_into()?))?;
    let mut ret_buffer = [0; 8];
    memory.read(&mut store, ret_ptr.try_into()?, &mut ret_buffer)?;
    let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
    let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

    Ok((bytecode_ptr, bytecode_len))
}

fn allocate_memory(
    instance: &Instance,
    mut store: &mut Store<WasiCtx>,
    alignment: u32,
    new_size: u32,
) -> Result<u32> {
    let realloc_func = instance
        .get_typed_func::<(u32, u32, u32, u32), u32, _>(&mut store, "canonical_abi_realloc")?;
    let orig_ptr = 0;
    let orig_size = 0;
    realloc_func
        .call(&mut store, (orig_ptr, orig_size, alignment, new_size))
        .map_err(Into::into)
}

fn compile_src_with_separate_instance(
    engine: &Engine,
    quickjs_provider_module: &Module,
    js_src: &str,
) -> Result<Vec<u8>> {
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().build();
    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(&mut store, &quickjs_provider_module)?;

    let (bytecode_ptr, bytecode_len) = compile_src(js_src.as_bytes(), &instance, &mut store)?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let mut bytecode = vec![0; bytecode_len.try_into()?];
    memory.read(&mut store, bytecode_ptr.try_into()?, &mut bytecode)?;
    Ok(bytecode)
}
