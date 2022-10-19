mod js_module;
mod options;

use crate::options::Options;
use anyhow::{Context, Result};
use std::convert::TryInto;
use std::fs;
use std::io::Read;
use structopt::StructOpt;
use wasmtime_wasi::WasiCtxBuilder;

fn main() -> Result<()> {
    let opts = Options::from_args();

    let mut contents = fs::File::open(&opts.input)
        .with_context(|| format!("Failed to open input file {}", opts.input.display()))?;
    let mut buffer = vec![];
    contents.read_to_end(&mut buffer)?;

    let core_wasm_module = "target/wasm32-wasi/release/javy_core.wasm";
    let engine = wasmtime::Engine::default();
    let mut linker = wasmtime::Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    let module = wasmtime::Module::from_file(&engine, core_wasm_module)?;
    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = wasmtime::Store::new(&engine, wasi);
    let instance = linker.instantiate(&mut store, &module)?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    let realloc = instance
        .get_typed_func::<(u32, u32, u32, u32), u32, _>(&mut store, "canonical_abi_realloc")?;
    let orig_ptr = 0;
    let existing_len = 0;

    let contents_alignment = 1;
    let contents_size = buffer.len();
    let contents_ptr = realloc.call(
        &mut store,
        (
            orig_ptr,
            existing_len,
            contents_alignment,
            contents_size.try_into()?,
        ),
    )?;

    let bytecode_len_ptr_alignment = 4;
    let bytecode_len_ptr_size = 1;
    let bytecode_len_ptr = realloc.call(
        &mut store,
        (
            orig_ptr,
            existing_len,
            bytecode_len_ptr_alignment,
            bytecode_len_ptr_size,
        ),
    )?;

    memory.write(&mut store, contents_ptr.try_into()?, &mut buffer)?;
    let bytecode_ptr = instance
        .get_typed_func::<(u32, u32, u32), u32, _>(&mut store, "compile-bytecode")?
        .call(
            &mut store,
            (contents_ptr, contents_size.try_into()?, bytecode_len_ptr),
        )?;

    let mut buffer = [0; 4];
    memory.read(&mut store, bytecode_len_ptr.try_into()?, &mut buffer)?;
    let bytecode_len = u32::from_le_bytes(buffer);

    let mut bytecode = vec![0; bytecode_len.try_into()?];
    memory.read(&store, bytecode_ptr.try_into()?, &mut bytecode)?;

    let module = js_module::JsModule::new(bytecode);
    let js_wat = module.to_wat();

    let js_wasm_binary = wat::parse_str(js_wat)?;

    fs::write(&opts.output, &js_wasm_binary)?;
    Ok(())
}
