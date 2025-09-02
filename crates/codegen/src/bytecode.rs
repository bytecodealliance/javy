use anyhow::{anyhow, bail, Result};
use wasmtime::{AsContext, AsContextMut, Engine, Instance, Linker, Memory, Module, Store};

use crate::{plugin::PluginKind, Plugin};

pub(crate) fn compile_source(
    plugin: &Plugin,
    plugin_kind: PluginKind,
    js_source_code: &[u8],
) -> Result<Vec<u8>> {
    let (mut store, instance, memory) = create_wasm_env(plugin.as_bytes())?;
    let (js_src_ptr, js_src_len) = copy_source_code_into_instance(
        js_source_code,
        store.as_context_mut(),
        &instance,
        &memory,
        plugin_kind,
    )?;
    let ret_ptr = call_compile(
        js_src_ptr,
        js_src_len,
        store.as_context_mut(),
        &instance,
        plugin_kind,
    )?;
    let bytecode =
        copy_bytecode_from_instance(ret_ptr, store.as_context_mut(), &memory, plugin_kind)?;
    Ok(bytecode)
}

fn create_wasm_env(plugin_bytes: &[u8]) -> Result<(Store<()>, Instance, Memory)> {
    let engine = Engine::default();
    let module = Module::new(&engine, plugin_bytes)?;
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    linker.define_unknown_imports_as_default_values(&mut store, &module)?;
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
    plugin_kind: PluginKind,
) -> Result<(u32, u32)> {
    let realloc_fn = instance.get_typed_func::<(u32, u32, u32, u32), u32>(
        store.as_context_mut(),
        plugin_kind.realloc_fn_name(),
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
    plugin_kind: PluginKind,
) -> Result<u32> {
    let compile_src_fn = instance
        .get_typed_func::<(u32, u32), u32>(store.as_context_mut(), plugin_kind.compile_fn_name())?;
    let ret_ptr = compile_src_fn
        .call(store.as_context_mut(), (js_src_ptr, js_src_len))
        .map_err(|e| anyhow!("JS compilation failed: {e}"))?;
    Ok(ret_ptr)
}

fn copy_bytecode_from_instance(
    ret_ptr: u32,
    store: impl AsContext,
    memory: &Memory,
    plugin_kind: PluginKind,
) -> Result<Vec<u8>> {
    let (bytecode_ptr, bytecode_len) = if plugin_kind == PluginKind::V2 {
        let mut ret_buffer = [0; 8];
        memory.read(store.as_context(), ret_ptr.try_into()?, &mut ret_buffer)?;
        let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
        let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);
        (bytecode_ptr, bytecode_len)
    } else {
        let mut ret_buffer = [0; 12];
        memory.read(store.as_context(), ret_ptr.try_into()?, &mut ret_buffer)?;

        let result = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
        let ptr = u32::from_le_bytes(ret_buffer[4..8].try_into()?);
        let len = u32::from_le_bytes(ret_buffer[8..12].try_into()?);
        // 0 is the result discriminator value for the success variant.
        if result != 0 {
            let mut error_bytes = vec![0; len as _];
            memory.read(store.as_context(), ptr as _, &mut error_bytes)?;
            let err = String::from_utf8_lossy(&error_bytes);
            bail!("Failed to compile source code: {err}")
        }
        (ptr, len)
    };

    let mut bytecode = vec![0; bytecode_len.try_into()?];
    memory.read(store.as_context(), bytecode_ptr.try_into()?, &mut bytecode)?;

    Ok(bytecode)
}
