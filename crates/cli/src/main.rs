mod bytecode;
mod codegen;
mod commands;
mod js;
mod wit;

use crate::codegen::{DynamicGenerator, StaticGenerator, WitOptions};
use crate::commands::{Cli, Command, EmitProviderCommandOpts};
use anyhow::Result;
use clap::Parser;
use codegen::CodeGenBuilder;
use js::JS;
use std::fs;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Command::EmitProvider(opts) => emit_provider(&opts),
        c @ Command::Compile(opts) | c @ Command::Build(opts) => {
            if c.is_compile() {
                println!(
                    r#"
                The `compile` command will be deprecated in the next major
                release of the CLI (v4.0.0)

                Refer to https://github.com/bytecodealliance/javy/issues/702 for
                details.
                
                Use the `build` command instead.
            "#
                );
            }

            let js = JS::from_file(&opts.input)?;
            let mut builder = CodeGenBuilder::new();
            builder
                .wit_opts(WitOptions::from_tuple((
                    opts.wit.clone(),
                    opts.wit_world.clone(),
                ))?)
                .source_compression(!opts.no_source_compression);

            builder.provider_version("2");

            let mut gen = if opts.dynamic {
                builder.build::<DynamicGenerator>()?
            } else {
                builder.build::<StaticGenerator>()?
            };

            let wasm = gen.generate(&js)?;

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
