mod bytecode;
mod codegen;
mod commands;
mod js;
mod option;
mod wit;

use crate::codegen::WitOptions;
use crate::commands::{Cli, Command, EmitProviderCommandOpts};
use anyhow::Result;
use bytecode::QUICKJS_PROVIDER_MODULE;
use clap::Parser;
use codegen::CodeGenBuilder;
use commands::{CodegenOptionGroup, JsOptionGroup};
use javy_config::Config;
use js::JS;
use std::fs;
use std::fs::File;
use std::io::Write;
use wizer::Wizer;

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
                .provider_version("2");

            let config = Config::default();
            let mut gen = if opts.dynamic {
                builder.build_dynamic(None)?
            } else {
                builder.build_static(config, None)?
            };

            let wasm = gen.generate(&js, QUICKJS_PROVIDER_MODULE)?;

            fs::write(&opts.output, wasm)?;
            Ok(())
        }
        Command::Build(opts) => {
            let js = JS::from_file(&opts.input)?;
            let codegen: CodegenOptionGroup = opts.codegen.clone().try_into()?;
            let mut builder = CodeGenBuilder::new();
            builder
                .wit_opts(codegen.wit)
                .source_compression(codegen.source_compression)
                .provider_version("3");

            let provider_module = if let Some(plugin) = opts.plugin.clone() {
                &fs::read(plugin)?
            } else {
                QUICKJS_PROVIDER_MODULE
            };
            let js_opts: JsOptionGroup = opts.js.clone().into();

            let mut gen = if codegen.dynamic {
                let import_namespace = bytecode::import_namespace(provider_module)?;
                builder.build_dynamic(Some(import_namespace.to_string()))?
            } else {
                builder.build_static(js_opts.into(), opts.plugin.clone())?
            };

            let wasm = gen.generate(&js, provider_module)?;

            fs::write(&opts.output, wasm)?;
            Ok(())
        }
        Command::InitializePlugin(opts) => {
            let wasm = fs::read(&opts.plugin)?;
            let wasm = Wizer::new()
                .allow_wasi(true)?
                .inherit_stdio(true)
                .init_func("initialize_runtime")
                .keep_init_func(true)
                .wasm_bulk_memory(true)
                .run(&wasm)?;
            fs::write(&opts.out, wasm)?;
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
