use std::env;
use std::fs;
use std::path::PathBuf;

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

    pub fn write_optimized_wasm(
        self,
        dest: impl AsRef<Path>,
        init_func: String,
    ) -> Result<(), Error> {
        let mut wasm = Wizer::new()
            .allow_wasi(true)
            .inherit_stdio(true)
            .init_func(init_func)
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

fn main() {
    if let Ok("cargo-clippy") = env::var("CARGO_CFG_FEATURE").as_ref().map(String::as_str) {
        stub_engine_for_clippy();
    } else {
        copy_engine_binary();
    }
}

// When using clippy, we need to write a stubbed engine.wasm file to ensure compilation succeeds. This
// skips building the actual engine.wasm binary that would be injected into the CLI binary.
fn stub_engine_for_clippy() {
    let engine_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("engine.wasm");

    if !engine_path.exists() {
        std::fs::write(engine_path, &[]).expect("failed to write empty engine.wasm stub");
        println!("cargo:warning=using stubbed engine.wasm for static analysis purposes...");
    }
}

// Copy the engine binary build from the `core` crate
fn copy_engine_binary() {
    let override_engine_path = env::var("JAVY_ENGINE_PATH");
    let is_override = override_engine_path.is_ok();
    let mut engine_path = PathBuf::from(
        override_engine_path.unwrap_or_else(|_| env::var("CARGO_MANIFEST_DIR").unwrap()),
    );

    if !is_override {
        engine_path.pop();
        engine_path.pop();
        engine_path = engine_path.join("target/wasm32-wasi/release/javy_core.wasm");
    }

    println!("cargo:rerun-if-changed={:?}", engine_path);
    println!("cargo:rerun-if-changed=build.rs");

    if engine_path.exists() {
        let copied_engine_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("engine.wasm");
        let engine_wasm = fs::read(&engine_path).expect("failed to read wasm module");
        Optimizer::new(engine_wasm.as_slice())
            .optimize(true)
            .write_optimized_wasm(&copied_engine_path, "init-engine".to_owned())
            .expect("failed to write optimized wasm");
    }
}
