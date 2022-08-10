use anyhow::{bail, Error, Result};
use binaryen::{CodegenConfig, Module};
use std::path::Path;
use wizer::Wizer;

pub(crate) struct Optimizer<'a> {
    optimize: bool,
    wasm: &'a [u8],
}

impl<'a> Optimizer<'a> {
    pub fn new(wasm: &'a [u8]) -> Self {
        Self {
            wasm,
            optimize: false,
        }
    }

    pub fn optimize(self, optimize: bool) -> Self {
        Self { optimize, ..self }
    }

    pub fn write_optimized_wasm(self, dest: impl AsRef<Path>) -> Result<(), Error> {
        let mut wasm = Wizer::new()
            .allow_wasi(true)?
            .inherit_stdio(true)
            .run(self.wasm)?;

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

        std::fs::write(dest.as_ref(), wasm)?;

        Ok(())
    }
}
