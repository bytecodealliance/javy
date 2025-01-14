mod commands;
mod option;

use crate::commands::{Cli, Command, EmitPluginCommandOpts};
use anyhow::Result;
use clap::Parser;
use codegen::builder::{CodeGenBuilder, WitOptions};
use codegen::js::JS;
use codegen::js_config::JsConfig;
use codegen::plugins::{Plugin, PluginKind, UninitializedPlugin};
use codegen::CodeGenType;
use commands::CliJsConfig;
use commands::CodegenOptionGroup;
use std::fs;
use std::fs::File;
use std::io::Write;

/// Use the default plugin for most commands.
const DEFAULT_PLUGIN_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.wasm"));

/// Use the legacy plugin when using the `compile -d` command.
const LEGACY_PLUGIN_MODULE: &[u8] = include_bytes!("./javy_quickjs_provider_v2.wasm");

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

            // Determine plugin configuration based on dynamic flag
            let (plugin_bytes, plugin_kind, gen_type) = if opts.dynamic {
                (LEGACY_PLUGIN_MODULE, PluginKind::V2, CodeGenType::Dynamic)
            } else {
                (
                    DEFAULT_PLUGIN_MODULE,
                    PluginKind::Default,
                    CodeGenType::Static,
                )
            };

            let plugin = Plugin::from_bytes(plugin_bytes, plugin_kind);
            let wit_options = WitOptions::from_tuple((opts.wit.clone(), opts.wit_world.clone()))?;

            let builder = CodeGenBuilder::new(plugin, wit_options, !opts.no_source_compression);

            let wasm = builder
                .build(gen_type, JsConfig::default())?
                .generate(&js)?;

            fs::write(&opts.output, wasm)?;
            Ok(())
        }
        Command::Build(opts) => {
            let js = JS::from_file(&opts.input)?;
            let codegen: CodegenOptionGroup = opts.codegen.clone().try_into()?;
            let codegen_type = if codegen.dynamic {
                CodeGenType::Dynamic
            } else {
                CodeGenType::Static
            };
            let plugin = match codegen.plugin {
                Some(path) => Plugin::new(&path, PluginKind::User)?,
                None => Plugin::from_bytes(DEFAULT_PLUGIN_MODULE, PluginKind::Default),
            };
            let js_opts = CliJsConfig::from_group_values(&plugin, opts.js.clone())?;

            let builder = CodeGenBuilder::new(plugin, codegen.wit, codegen.source_compression);

            let wasm = builder.build(codegen_type, js_opts)?.generate(&js)?;

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
    file.write_all(Plugin::from_bytes(DEFAULT_PLUGIN_MODULE, PluginKind::Default).as_bytes())?;
    Ok(())
}
