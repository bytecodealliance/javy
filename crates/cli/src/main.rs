mod js_module;
mod options;

use crate::options::Options;
use anyhow::{Context, Result};
use std::convert::TryInto;
use std::fs;
use std::io::Read;
use structopt::StructOpt;

struct MyState {}

fn main() -> Result<()> {
    let opts = Options::from_args();

    let mut contents = fs::File::open(&opts.input)
        .with_context(|| format!("Failed to open input file {}", opts.input.display()))?;
    let mut buffer = vec![];
    contents.read_to_end(&mut buffer);

    let core_wasm_module = "target/wasm32-wasi/release/javy_core.wasm";
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_file(&engine, core_wasm_module)?;
    let mut store = wasmtime::Store::new(&engine, MyState {});
    let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

    let orig_ptr = 0;
    let existing_len = 0;
    let alignment = 1;
    let contents_size = buffer.len();
    let contents_ptr = instance.get_typed_func::<(u32, u32, u32, u32), u32, _>(&mut store, "canonical_abi_realloc")?.call(&mut store, (orig_ptr, existing_len, alignment, contents_size.try_into().unwrap()))?;

    let run = instance.get_typed_func::<(u32, u32, u32), u32, _>(&mut store, "compile-bytecode")?;
    let mut bytecode_len = 0;
    let bytecode_ptr = run.call(&mut store, (contents_ptr, contents_size.try_into().unwrap(), bytecode_len))?;
    // FIXME need to convert bytecode_ptr to process address (is currently the linear memory address)
    let bytecode = unsafe { std::vec::Vec::from_raw_parts(bytecode_ptr, bytecode_len as usize, bytecode_len as usize) };

    // let bytecode = fs::read("index.kbc1")?;
    // let bytecode = fs::read("/Users/jeffcharles/projects/convert-c-hex-to-binary/output")?;
    let module = js_module::JsModule::new(bytecode);
    let js_wat = module.to_wat();

    let js_wasm_binary = wat::parse_str(js_wat)?;

    fs::write(&opts.output, &js_wasm_binary)?;
    Ok(())
}
