mod bytecode;
mod codegen;
mod commands;
mod js;
mod option;
mod providers;
mod runtime_config;
mod wit;

use crate::codegen::WitOptions;
use crate::commands::{Cli, Command, EmitProviderCommandOpts};
use anyhow::Result;
use clap::Parser;
use codegen::{CodeGenBuilder, CodeGenType};
use commands::{CodegenOptionGroup, JsOptionGroup};
use js::JS;
use providers::Provider;
use runtime_config::Config;
use std::fs;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Command::EmitProvider(opts) => emit_provider(opts),
        Command::Compile(opts) => {
            eprintln!(
                r#"
                The `compile` command will be deprecated in the next major
                release of the CLI (v4.0.0)

                Refer to https://github.com/bytecodealliance/javy/issues/702 for
                details.
                
                Use the `build` command instead.
            "#
            );

            let js = JS::from_file(&opts.input)?;
            let mut builder = CodeGenBuilder::new();
            builder
                .wit_opts(WitOptions::from_tuple((
                    opts.wit.clone(),
                    opts.wit_world.clone(),
                ))?)
                .source_compression(!opts.no_source_compression)
                .provider(Provider::V2);

            let config = Config::default();
            let mut gen = if opts.dynamic {
                builder.build(CodeGenType::Dynamic, config)?
            } else {
                builder.build(CodeGenType::Static, config)?
            };

            let wasm = gen.generate(&js)?;

            fs::write(&opts.output, wasm)?;
            Ok(())
        }
        Command::Build(opts) => {
            let js = JS::from_file(&opts.input)?;
            let codegen: CodegenOptionGroup = opts.codegen.clone().try_into()?;
            let mut builder = CodeGenBuilder::new();
            builder
                .wit_opts(codegen.wit)
                .source_compression(codegen.source_compression);

            let js_opts: JsOptionGroup = opts.js.clone().into();
            let mut gen = if codegen.dynamic {
                builder.build(CodeGenType::Dynamic, js_opts.into())?
            } else {
                builder.build(CodeGenType::Static, js_opts.into())?
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
    file.write_all(Provider::Default.as_bytes())?;
    Ok(())
}
