mod js;
mod options;

use crate::options::Options;
use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use structopt::StructOpt;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

fn main() -> Result<()> {
    let opts = Options::from_args();

    let mut contents = fs::File::open(&opts.input)
        .with_context(|| format!("Failed to open input file {}", opts.input.display()))?;

    let mut js_text = String::new();
    contents.read_to_string(&mut js_text).unwrap();
    let module = js::JsModule::new(&js_text);
    let js_wat = module.to_wat();

    println!("{}", &js_wat);

    let wasm_binary = wat::parse_str(js_wat)?;
    fs::write(&opts.output, &wasm_binary)?;

    let core_js_wasm = include_bytes!("../javy_core.wizened.wasm");

    let mut wasm_config = wasmtime::Config::new();
    wasm_config
        .wasm_module_linking(true)
        .wasm_multi_memory(true);
    let engine = Engine::new(&wasm_config)?;

    // First set up our linker which is going to be linking modules together. We
    // want our linker to have wasi available, so we set that up here as well.
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Load and compile our two modules
    let core_module = Module::from_binary(&engine, core_js_wasm)?;
    let js_module = Module::from_binary(&engine, &wasm_binary)?;

    // Configure WASI and insert it into a `Store`
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our first module which only uses WASI, then register that
    // instance with the linker since the next linking will use it.
    let core_instance = linker.instantiate(&mut store, &core_module)?;
    linker.instance(&mut store, "js_engine", core_instance)?;

    // And with that we can perform the final link and the execute the module.
    let js_instance = linker.instantiate(&mut store, &js_module)?;
    let run = js_instance.get_typed_func::<(), (), _>(&mut store, "shopify_main")?;
    run.call(&mut store, ())?;

    Ok(())
}
