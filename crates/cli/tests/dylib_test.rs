use anyhow::Result;
use std::boxed::Box;
use std::str;
use wasi_common::pipe::WritePipe;
use wasmtime::{Engine, Instance, Linker, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

mod common;

#[test]
fn test_dylib() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let stderr = WritePipe::new_in_memory();
    let wasi = WasiCtxBuilder::new()
        .stderr(Box::new(stderr.clone()))
        .build();
    let module = common::create_quickjs_provider_module(&engine)?;

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
