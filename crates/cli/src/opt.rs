use anyhow::{bail, Context, Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use wizer::Wizer;

use crate::prebuilt;

pub(crate) struct Optimizer<'a> {
    strip: bool,
    optimize: bool,
    script: PathBuf,
    wasm: &'a [u8],
}

impl<'a> Optimizer<'a> {
    pub fn new(wasm: &'a [u8], script: PathBuf) -> Self {
        Self {
            wasm,
            script,
            strip: false,
            optimize: false,
        }
    }

    pub fn strip(self, strip: bool) -> Self {
        Self { strip, ..self }
    }

    pub fn optimize(self, optimize: bool) -> Self {
        Self { optimize, ..self }
    }

    pub fn write_optimized_wasm(self, dest: impl AsRef<Path>) -> Result<(), Error> {
        let dir = self
            .script
            .parent()
            .filter(|p| p.is_dir())
            .context("input script is not a file")?;

        let wasm = Wizer::new()
            .allow_wasi(true)
            .inherit_env(true)
            .dir(dir)
            .run(self.wasm)?;

        std::fs::write(dest.as_ref(), wasm)?;

        if self.strip {
            let output = Command::new(prebuilt::wasm_strip())
                .arg(dest.as_ref())
                .output()?;

            if !output.status.success() {
                bail!(format!("Couldn't apply wasm-strip: {:?}", output.stderr));
            }
        }

        if self.optimize {
            let output = Command::new(prebuilt::wasm_opt())
                .arg(dest.as_ref())
                .arg("-O3")
                .arg("--dce")
                .arg("-o")
                .arg(dest.as_ref())
                .output()?;

            if !output.status.success() {
                bail!(format!("Couldn't apply wasm-opt: {:?}", output.stderr));
            }
        }

        Ok(())
    }
}
