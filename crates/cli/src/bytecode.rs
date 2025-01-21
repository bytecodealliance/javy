use anyhow::{anyhow, Result};
use wasi_common::{sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{AsContextMut, Engine, Instance, Linker, Memory, Module, Store};

use crate::plugins::Plugin;

pub fn compile_source(plugin: &Plugin, js_source_code: &[u8]) -> Result<Vec<u8>> {
    let (mut store, instance, memory) = create_wasm_env(plugin)?;
    let (js_src_ptr, js_src_len) =
        copy_source_code_into_instance(js_source_code, store.as_context_mut(), &instance, &memory)?;
    let ret_ptr = call_compile(js_src_ptr, js_src_len, store.as_context_mut(), &instance)?;
    let bytecode = copy_bytecode_from_instance(ret_ptr, store.as_context_mut(), &memory)?;
    Ok(bytecode)
}

fn create_wasm_env(plugin: &Plugin) -> Result<(Store<WasiCtx>, Instance, Memory)> {
    let engine = Engine::default();
    let module = Module::new(&engine, plugin.as_bytes())?;
    let mut linker = Linker::new(&engine);
    wasi_common::sync::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker(
        &mut linker,
        |s| s,
    )?;
    linker.define_unknown_imports_as_traps(&module)?;
    let wasi = WasiCtxBuilder::new().inherit_stderr().build();
    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(store.as_context_mut(), &module)?;
    let memory = instance
        .get_memory(store.as_context_mut(), "memory")
        .unwrap();
    Ok((store, instance, memory))
}

fn copy_source_code_into_instance(
    js_source_code: &[u8],
    mut store: impl AsContextMut,
    instance: &Instance,
    memory: &Memory,
) -> Result<(u32, u32)> {
    let realloc_fn = instance.get_typed_func::<(u32, u32, u32, u32), u32>(
        store.as_context_mut(),
        "canonical_abi_realloc",
    )?;
    let js_src_len = js_source_code.len().try_into()?;

    let original_ptr = 0;
    let original_size = 0;
    let alignment = 1;
    let size = js_src_len;
    let js_source_ptr = realloc_fn.call(
        store.as_context_mut(),
        (original_ptr, original_size, alignment, size),
    )?;

    memory.write(
        store.as_context_mut(),
        js_source_ptr.try_into()?,
        js_source_code,
    )?;

    Ok((js_source_ptr, js_src_len))
}

fn call_compile(
    js_src_ptr: u32,
    js_src_len: u32,
    mut store: impl AsContextMut,
    instance: &Instance,
) -> Result<u32> {
    let compile_src_fn =
        instance.get_typed_func::<(u32, u32), u32>(store.as_context_mut(), "compile_src")?;
    let ret_ptr = compile_src_fn
        .call(store.as_context_mut(), (js_src_ptr, js_src_len))
        .map_err(|_| anyhow!("JS compilation failed"))?;
    Ok(ret_ptr)
}

fn copy_bytecode_from_instance(
    ret_ptr: u32,
    mut store: impl AsContextMut,
    memory: &Memory,
) -> Result<Vec<u8>> {
    let mut ret_buffer = [0; 8];
    memory.read(store.as_context_mut(), ret_ptr.try_into()?, &mut ret_buffer)?;

    let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
    let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

    let mut bytecode = vec![0; bytecode_len.try_into()?];
    memory.read(store.as_context(), bytecode_ptr.try_into()?, &mut bytecode)?;

    Ok(bytecode)
}
