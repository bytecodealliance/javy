use anyhow::{anyhow, bail, Result};
use binaryen::{CodegenConfig, Module};
use wizer::Wizer;

use crate::js::JS;

use super::transform::{self, SourceCodeSection};

/// Generates Wasm for a static Javy module within a subprocess.
///
/// We assume stdin contains the JS source code.
pub struct Generator {}

impl Generator {
    pub fn new() -> Generator {
        Generator {}
    }

    pub fn generate(&self) -> Result<Vec<u8>> {
        let wasm = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
        let wasm = Wizer::new()
            .allow_wasi(true)?
            .inherit_stdio(true)
            .wasm_bulk_memory(true)
            .run(wasm)
            .map_err(|_| anyhow!("JS compilation failed"))?;
        Ok(wasm)
    }
}

/// Takes Wasm created by `Generator` and makes additional changes.
///
/// This is intended to be run in the parent process after generating the Wasm.
pub struct Refiner<'a> {
    optimize: bool,
    js: &'a JS,
}

impl<'a> Refiner<'a> {
    pub fn new(js: &'a JS) -> Self {
        Self {
            optimize: false,
            js,
        }
    }

    pub fn optimize(self, optimize: bool) -> Self {
        Self { optimize, ..self }
    }

    pub fn refine(&self, mut wasm: Vec<u8>) -> Result<Vec<u8>> {
        if self.optimize {
            let codegen_cfg = CodegenConfig {
                optimization_level: 3, // Aggressively optimize for speed.
                shrink_level: 0,       // Don't optimize for size at the expense of performance.
                debug_info: false,
            };

            if let Ok(mut module) = Module::read(&wasm) {
                module.optimize(&codegen_cfg);
                module
                    .run_optimization_passes(vec!["strip"], &codegen_cfg)
                    .unwrap();
                wasm = module.write();
            } else {
                bail!("Unable to read wasm binary for wasm-opt optimizations");
            }
        }

        let mut module = transform::module_config().parse(&wasm)?;
        module.customs.add(SourceCodeSection::new(self.js)?);
        transform::add_producers_section(&mut module.producers);
        Ok(module.emit_wasm())
    }
}
