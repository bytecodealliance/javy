use anyhow::Result;
use wasmtime::{
    component::{bindgen, Component, Linker},
    AsContextMut, Engine, Store,
};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

bindgen!({
    inline: r#"
package bytecodealliance:javy-plugin;

interface javy-plugin-exports {
    compile-src: func(bytecode: list<u8>) -> list<u8>;
}

world javy {
    export javy-plugin-exports;
}
    "#
});

struct Wrapper {
    wasi: WasiCtx,
    resource_table: ResourceTable,
}

impl WasiView for Wrapper {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl IoView for Wrapper {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

pub(crate) fn compile_source(plugin_bytes: &[u8], js_source_code: &[u8]) -> Result<Vec<u8>> {
    let engine = Engine::default();
    let component = Component::new(&engine, plugin_bytes)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    // Error: map entry `wasi:cli/environment@0.2.3` defined twice
    // linker.define_unknown_imports_as_traps(&component)?;
    let wasi = WasiCtxBuilder::new().inherit_stderr().build();
    let mut store = Store::new(
        &engine,
        Wrapper {
            wasi,
            resource_table: ResourceTable::new(),
        },
    );
    let instance = Javy::instantiate(store.as_context_mut(), &component, &linker)?;
    let bytecode = instance
        .bytecodealliance_javy_plugin_javy_plugin_exports()
        .call_compile_src(store.as_context_mut(), js_source_code)?;
    Ok(bytecode)
}

// fn create_wasm_env(plugin_bytes: &[u8]) -> Result<(Store<WasiP1Ctx>, Instance, Memory)> {
//     let engine = Engine::default();
//     let component = Component::new(&engine, plugin_bytes)?;
//     let mut linker = Linker::new(&engine);
//     wasmtime_wasi::add_to_linker_sync(&mut linker)?;
//     linker.define_unknown_imports_as_traps(&component)?;
//     let wasi = WasiCtxBuilder::new().inherit_stderr().build_p1();
//     let mut store = Store::new(&engine, wasi);
//     let instance = linker.instantiate(store.as_context_mut(), &component)?;
//     let memory = instance
//         .get_memory(store.as_context_mut(), "memory")
//         .unwrap();
//     Ok((store, instance, memory))
// }

// fn copy_source_code_into_instance(
//     js_source_code: &[u8],
//     mut store: impl AsContextMut,
//     instance: &Instance,
//     memory: &Memory,
// ) -> Result<(u32, u32)> {
//     let realloc_fn = instance.get_typed_func::<(u32, u32, u32, u32), u32>(
//         store.as_context_mut(),
//         "canonical_abi_realloc",
//     )?;
//     let js_src_len = js_source_code.len().try_into()?;

//     let original_ptr = 0;
//     let original_size = 0;
//     let alignment = 1;
//     let size = js_src_len;
//     let js_source_ptr = realloc_fn.call(
//         store.as_context_mut(),
//         (original_ptr, original_size, alignment, size),
//     )?;

//     memory.write(
//         store.as_context_mut(),
//         js_source_ptr.try_into()?,
//         js_source_code,
//     )?;

//     Ok((js_source_ptr, js_src_len))
// }

// fn call_compile(
//     js_src_ptr: u32,
//     js_src_len: u32,
//     mut store: impl AsContextMut,
//     instance: &Instance,
// ) -> Result<u32> {
//     let compile_src_fn =
//         instance.get_typed_func::<(u32, u32), u32>(store.as_context_mut(), "compile_src")?;
//     let ret_ptr = compile_src_fn
//         .call(store.as_context_mut(), (js_src_ptr, js_src_len))
//         .map_err(|_| anyhow!("JS compilation failed"))?;
//     Ok(ret_ptr)
// }

// fn copy_bytecode_from_instance(
//     ret_ptr: u32,
//     mut store: impl AsContextMut,
//     memory: &Memory,
// ) -> Result<Vec<u8>> {
//     let mut ret_buffer = [0; 8];
//     memory.read(store.as_context_mut(), ret_ptr.try_into()?, &mut ret_buffer)?;

//     let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
//     let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

//     let mut bytecode = vec![0; bytecode_len.try_into()?];
//     memory.read(store.as_context(), bytecode_ptr.try_into()?, &mut bytecode)?;

//     Ok(bytecode)
// }
