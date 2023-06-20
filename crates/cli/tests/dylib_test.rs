use anyhow::Result;
use std::boxed::Box;
use std::str;
use wasi_common::{pipe::WritePipe, WasiFile};
use wasmtime::{Engine, Instance, Linker, Store};
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

mod common;

#[test]
fn test_dylib() -> Result<()> {
    let js_src = "console.log(42);";
    let stderr = WritePipe::new_in_memory();
    run_js_src(js_src, &stderr)?;

    let output = stderr.try_into_inner().unwrap().into_inner();
    assert_eq!("42\n", str::from_utf8(&output)?);

    Ok(())
}

#[test]
fn test_dylib_with_error() -> Result<()> {
    let js_src = "function foo() { throw new Error('foo error'); } foo();";
    let stderr = WritePipe::new_in_memory();
    let result = run_js_src(js_src, &stderr);

    assert!(result.is_err());
    let output = stderr.try_into_inner().unwrap().into_inner();

    let expected_log_output = "Error while running JS: Uncaught Error: foo error\n    at foo (function.mjs)\n    at <anonymous> (function.mjs:1)\n\n";
    assert_eq!(expected_log_output, str::from_utf8(&output)?);

    Ok(())
}

#[test]
fn test_dylib_with_exported_func() -> Result<()> {
    let js_src = "export function foo() { console.log('In foo'); }; console.log('Toplevel');";
    let stderr = WritePipe::new_in_memory();
    run_invoke(js_src, "foo", &stderr)?;

    let output = stderr.try_into_inner().unwrap().into_inner();
    assert_eq!("Toplevel\nIn foo\n", str::from_utf8(&output)?);

    Ok(())
}

fn run_js_src<T: WasiFile + Clone + 'static>(js_src: &str, stderr: &T) -> Result<()> {
    let (instance, mut store) = create_wasm_env(stderr)?;

    let eval_bytecode_func =
        instance.get_typed_func::<(u32, u32), ()>(&mut store, "eval_bytecode")?;
    let (bytecode_ptr, bytecode_len) = compile_src(js_src.as_bytes(), &instance, &mut store)?;
    eval_bytecode_func.call(&mut store, (bytecode_ptr, bytecode_len))?;
    Ok(())
}

fn run_invoke<T: WasiFile + Clone + 'static>(
    js_src: &str,
    fn_to_invoke: &str,
    stderr: &T,
) -> Result<()> {
    let (instance, mut store) = create_wasm_env(stderr)?;

    let invoke_func = instance.get_typed_func::<(u32, u32, u32, u32), ()>(&mut store, "invoke")?;
    let (bytecode_ptr, bytecode_len) = compile_src(js_src.as_bytes(), &instance, &mut store)?;
    let (fn_name_ptr, fn_name_len) = copy_func_name(fn_to_invoke, &instance, &mut store)?;
    invoke_func.call(
        &mut store,
        (bytecode_ptr, bytecode_len, fn_name_ptr, fn_name_len),
    )?;
    Ok(())
}

fn create_wasm_env<T: WasiFile + Clone + 'static>(
    stderr: &T,
) -> Result<(Instance, Store<WasiCtx>)> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let wasi = WasiCtxBuilder::new()
        .stderr(Box::new(stderr.clone()))
        .build();
    let module = common::create_quickjs_provider_module(&engine)?;

    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(&mut store, &module)?;

    Ok((instance, store))
}

fn compile_src(
    js_src: &[u8],
    instance: &Instance,
    mut store: &mut Store<WasiCtx>,
) -> Result<(u32, u32)> {
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let compile_src_func = instance.get_typed_func::<(u32, u32), u32>(&mut store, "compile_src")?;

    let js_src_ptr = allocate_memory(instance, store, 1, js_src.len().try_into()?)?;
    memory.write(&mut store, js_src_ptr.try_into()?, js_src)?;

    let ret_ptr = compile_src_func.call(&mut store, (js_src_ptr, js_src.len().try_into()?))?;
    let mut ret_buffer = [0; 8];
    memory.read(&mut store, ret_ptr.try_into()?, &mut ret_buffer)?;
    let bytecode_ptr = u32::from_le_bytes(ret_buffer[0..4].try_into()?);
    let bytecode_len = u32::from_le_bytes(ret_buffer[4..8].try_into()?);

    Ok((bytecode_ptr, bytecode_len))
}

fn copy_func_name(
    fn_name: &str,
    instance: &Instance,
    mut store: &mut Store<WasiCtx>,
) -> Result<(u32, u32)> {
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let fn_name_bytes = fn_name.as_bytes();
    let fn_name_ptr = allocate_memory(instance, store, 1, fn_name_bytes.len().try_into()?)?;
    memory.write(&mut store, fn_name_ptr.try_into()?, fn_name_bytes)?;

    Ok((fn_name_ptr, fn_name_bytes.len().try_into()?))
}

fn allocate_memory(
    instance: &Instance,
    mut store: &mut Store<WasiCtx>,
    alignment: u32,
    new_size: u32,
) -> Result<u32> {
    let realloc_func = instance
        .get_typed_func::<(u32, u32, u32, u32), u32>(&mut store, "canonical_abi_realloc")?;
    let orig_ptr = 0;
    let orig_size = 0;
    realloc_func
        .call(&mut store, (orig_ptr, orig_size, alignment, new_size))
        .map_err(Into::into)
}
