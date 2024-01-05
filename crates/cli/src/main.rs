mod bytecode;
mod commands;
mod exports;
mod js;
mod wasm_generator;
mod wit;

use crate::commands::{Command, EmitProviderCommandOpts};
use crate::wasm_generator::r#static as static_generator;
use anyhow::{bail, Result};
use js::JS;
use std::fs;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;
use wasm_generator::dynamic as dynamic_generator;

fn main() -> Result<()> {
    let cmd = Command::from_args();

    match &cmd {
        Command::EmitProvider(opts) => emit_provider(opts),
        Command::Compile(opts) => {
            let js = JS::from_file(&opts.input)?;
            let exports = match (&opts.wit, &opts.wit_world) {
                (None, None) => Ok(vec![]),
                (None, Some(_)) => Ok(vec![]),
                (Some(_), None) => bail!("Must provide WIT world when providing WIT file"),
                (Some(wit), Some(world)) => exports::process_exports(&js, wit, world),
            }?;
            let wasm = if opts.dynamic {
                dynamic_generator::generate(&js, exports, opts.no_source_compression)?
            } else {
                static_generator::generate(&js, exports, opts.no_source_compression)?
            };
            fs::write(&opts.output, wasm)?;
            Ok(())
        }
    }
}

fn emit_provider(opts: &EmitProviderCommandOpts) -> Result<()> {
    let mut file: Box<dyn Write> = match opts.out.as_ref() {
        Some(path) => Box::new(File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };
    file.write_all(bytecode::QUICKJS_PROVIDER_MODULE)?;
    Ok(())
}
