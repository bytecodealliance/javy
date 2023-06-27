use anyhow::{anyhow, Result};
use binaryen::{CodegenConfig, Module};
use wizer::Wizer;

use crate::js::JS;

use super::transform::{self, SourceCodeSection};

/// Generates Wasm for a static Javy module within a subprocess.
///
/// We assume stdin contains the JS source code.
pub fn generate() -> Result<Vec<u8>> {
    let wasm = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
    let wasm = Wizer::new()
        .allow_wasi(true)?
        .inherit_stdio(true)
        .wasm_bulk_memory(true)
        .run(wasm)
        .map_err(|_| anyhow!("JS compilation failed"))?;
    Ok(wasm)
}

/// Takes Wasm created by `Generator` and makes additional changes.
///
/// This is intended to be run in the parent process after generating the Wasm.
pub fn refine(mut wasm: Vec<u8>, js: &JS) -> Result<Vec<u8>> {
    let codegen_cfg = CodegenConfig {
        optimization_level: 3, // Aggressively optimize for speed.
        shrink_level: 0,       // Don't optimize for size at the expense of performance.
        debug_info: false,
    };

    let mut module = Module::read(&wasm)
        .map_err(|_| anyhow!("Unable to read wasm binary for wasm-opt optimizations"))?;
    module.optimize(&codegen_cfg);
    module
        .run_optimization_passes(vec!["strip"], &codegen_cfg)
        .map_err(|_| anyhow!("Running wasm-opt optimization passes failed"))?;
    wasm = module.write();

    let mut module = transform::module_config().parse(&wasm)?;
    module.customs.add(SourceCodeSection::new(js)?);
    transform::add_producers_section(&mut module.producers);
    Ok(module.emit_wasm())
}
