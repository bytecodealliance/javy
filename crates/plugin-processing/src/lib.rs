use anyhow::Result;
use std::{fs, rc::Rc};
use wasmtime_wasi::WasiCtxBuilder;
use wizer::{wasmtime::Module, Linker, Wizer};

/// Uses wasm-opt and Wizer to initialize a plugin.
pub fn initialize_plugin(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    // Re-encode overlong indexes with wasm-opt before running Wizer.
    let wasm_bytes = optimize_module(wasm_bytes)?;
    let wasm_bytes = preinitialize_module(&wasm_bytes)?;
    Ok(wasm_bytes)
}

fn optimize_module(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let temp_dir = tempfile::tempdir()?;
    let infile = temp_dir.path().join("infile.wasm");
    fs::write(&infile, wasm_bytes)?;
    let outfile = temp_dir.path().join("outfile.wasm");
    wasm_opt::OptimizationOptions::new_opt_level_3() // Aggressively optimize for speed.
        .shrink_level(wasm_opt::ShrinkLevel::Level0) // Don't optimize for size at the expense of performance.
        .debug_info(false)
        .run(&infile, &outfile)?;
    let optimized_wasm_bytes = fs::read(outfile)?;
    Ok(optimized_wasm_bytes)
}

fn preinitialize_module(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut wizer = Wizer::new();
    let owned_wasm_bytes = wasm_bytes.to_vec();
    wizer
        .init_func("initialize_runtime")
        .keep_init_func(true)
        .make_linker(Some(Rc::new(move |engine| {
            let mut linker = Linker::new(engine);
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |ctx| {
                if ctx.wasi_ctx.is_none() {
                    ctx.wasi_ctx = Some(WasiCtxBuilder::new().inherit_stderr().build_p1());
                }
                ctx.wasi_ctx.as_mut().unwrap()
            })?;
            linker.define_unknown_imports_as_traps(&Module::new(engine, &owned_wasm_bytes)?)?;
            Ok(linker)
        })))?
        .run(wasm_bytes)
}
