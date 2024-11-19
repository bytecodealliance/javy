mod bytecode;
mod codegen;
mod commands;
mod js;
mod js_config;
mod option;
mod plugins;
mod wit;

use crate::codegen::WitOptions;
use crate::commands::{Cli, Command, EmitPluginCommandOpts};
use anyhow::Result;
use clap::Parser;
use codegen::{CodeGenBuilder, CodeGenType};
use commands::CodegenOptionGroup;
use js::JS;
use js_config::JsConfig;
use plugins::{Plugin, UninitializedPlugin};
use std::fs;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Command::EmitPlugin(opts) => emit_plugin(opts),
        Command::Compile(opts) => {
            eprintln!(
                r#"
                The `compile` command is deprecated and will be removed.

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
                .source_compression(!opts.no_source_compression);

            let config = JsConfig::default();
            let mut gen = if opts.dynamic {
                builder.plugin(Plugin::V2);
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
            let plugin = match codegen.plugin {
                Some(path) => Plugin::new_user_plugin(&path)?,
                None => Plugin::Default,
            };
            let js_opts = JsConfig::from_group_values(&plugin, opts.js.clone())?;
            let mut builder = CodeGenBuilder::new();
            builder
                .wit_opts(codegen.wit)
                .source_compression(codegen.source_compression)
                .plugin(plugin);
            let mut gen = if codegen.dynamic {
                builder.build(CodeGenType::Dynamic, js_opts)?
            } else {
                builder.build(CodeGenType::Static, js_opts)?
            };

            let wasm = gen.generate(&js)?;

            fs::write(&opts.output, wasm)?;
            Ok(())
        }
        Command::InitPlugin(opts) => {
            let plugin_bytes = fs::read(&opts.plugin)?;

            let uninitialized_plugin = UninitializedPlugin::new(&plugin_bytes)?;
            let initialized_plugin_bytes = uninitialized_plugin.initialize()?;

            let mut out: Box<dyn Write> = match opts.out.as_ref() {
                Some(path) => Box::new(File::create(path)?),
                None => Box::new(std::io::stdout()),
            };
            out.write_all(&initialized_plugin_bytes)?;
            Ok(())
        }
    }
}

fn emit_plugin(opts: &EmitPluginCommandOpts) -> Result<()> {
    let mut file: Box<dyn Write> = match opts.out.as_ref() {
        Some(path) => Box::new(File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };
    file.write_all(Plugin::Default.as_bytes())?;
    Ok(())
}
