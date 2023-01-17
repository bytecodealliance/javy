use anyhow::Result;
use wasmtime::{Engine, Instance, Linker, Memory, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

const QUICKJS_PROVIDER_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/provider.wasm"));

pub fn compile_source(js_source_code: &[u8]) -> Result<Vec<u8>> {
    let (mut store, instance, memory) = create_wasm_env()?;
    let (js_src_ptr, js_src_len) =
        copy_source_code_into_instance(js_source_code, &mut store, &instance, &memory)?;
    let ret_ptr = call_compile(js_src_ptr, js_src_len, &mut store, &instance)?;
    let bytecode = copy_bytecode_from_instance(ret_ptr, &mut store, &memory)?;
    Ok(bytecode)
}

fn create_wasm_env() -> Result<(Store<WasiCtx>, Instance, Memory)> {
    let engine = Engine::default();
    let module = Module::new(&engine, QUICKJS_PROVIDER_MODULE)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new().build();
    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(&mut store, &module)?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    Ok((store, instance, memory))
}

fn copy_source_code_into_instance(
    js_source_code: &[u8],
    mut store: &mut Store<WasiCtx>,
    instance: &Instance,
    memory: &Memory,
) -> Result<(u32, u32)> {
    let realloc_fn = instance
        .get_typed_func::<(u32, u32, u32, u32), u32, _>(&mut store, "canonical_abi_realloc")?;
    let js_src_len = js_source_code.len().try_into()?;

    let original_ptr = 0;
    let original_size = 0;
    let alignment = 1;
    let size = js_src_len;
    let js_source_ptr =
        realloc_fn.call(&mut store, (original_ptr, original_size, alignment, size))?;

    memory.write(&mut store, js_source_ptr.try_into()?, js_source_code)?;

    Ok((js_source_ptr, js_src_len))
}

fn call_compile(
    js_src_ptr: u32,
    js_src_len: u32,
    mut store: &mut Store<WasiCtx>,
    instance: &Instance,
) -> Result<u32> {
    let compile_src_fn =
        instance.get_typed_func::<(u32, u32), u32, _>(&mut store, "compile_src")?;
    let ret_ptr = compile_src_fn.call(&mut store, (js_src_ptr, js_src_len))?;
    Ok(ret_ptr)
}

fn copy_bytecode_from_instance(
    ret_ptr: u32,
    mut store: &mut Store<WasiCtx>,
    memory: &Memory,
) -> Result<Vec<u8>> {
    let mut ret_buffer = [0; 8];
    memory.read(&mut store, ret_ptr.try_into()?, &mut ret_buffer)?;

    let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
    let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

    let mut bytecode = vec![0; bytecode_len.try_into()?];
    memory.read(&mut store, bytecode_ptr.try_into()?, &mut bytecode)?;

    Ok(bytecode)
}
