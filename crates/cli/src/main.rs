mod commands;
mod js_config;
mod option;
mod plugin;

use crate::commands::{Cli, Command, EmitPluginCommandOpts};
use anyhow::Result;
use clap::Parser;

use commands::CodegenOptionGroup;
use javy_codegen::{Generator, LinkingKind, Plugin, WitOptions, JS};
use js_config::JsConfig;
use plugin::{
    CliPlugin, PluginKind, UninitializedPlugin, PLUGIN_MODULE, QUICKJS_PROVIDER_V2_MODULE,
};
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

            let plugin_bytes = if opts.dynamic {
                QUICKJS_PROVIDER_V2_MODULE
            } else {
                PLUGIN_MODULE
            };

            let mut generator = Generator::new(Plugin::new(plugin_bytes.into()));

            if opts.dynamic {
                generator
                    .linking(LinkingKind::Dynamic)
                    .linking_v2_plugin(true);
            } else {
                generator
                    .linking(LinkingKind::Static)
                    .linking_default_plugin(true);
            }

            generator
                .wit_opts(WitOptions::from_tuple((
                    opts.wit.clone(),
                    opts.wit_world.clone(),
                ))?)
                .source_compression(!opts.no_source_compression)
                .js_runtime_config(JsConfig::default().to_json()?);

            let wasm = generator.generate(&js)?;

            fs::write(&opts.output, wasm)?;
            Ok(())
        }
        Command::Build(opts) => {
            let js = JS::from_file(&opts.input)?;
            let codegen_opts: CodegenOptionGroup = opts.codegen.clone().try_into()?;

            // Always assume the default plugin if no plugin is provided.
            let cli_plugin = match &codegen_opts.plugin {
                Some(path) => CliPlugin::new(Plugin::new_from_path(path)?, PluginKind::User),
                None => CliPlugin::new(Plugin::new(PLUGIN_MODULE.into()), PluginKind::Default),
            };

            let js_opts = JsConfig::from_group_values(&cli_plugin, opts.js.clone())?;

            let mut generator = Generator::new(cli_plugin.into_plugin());

            // Always link to the default plugin if no plugin is provided.
            if codegen_opts.plugin.is_none() {
                generator.linking_default_plugin(true);
            }

            // Configure the generator with the provided options.
            generator
                .wit_opts(codegen_opts.wit)
                .source_compression(!codegen_opts.source_compression)
                .js_runtime_config(js_opts.to_json()?);

            if codegen_opts.dynamic {
                generator.linking(LinkingKind::Dynamic);
            } else {
                generator.linking(LinkingKind::Static);
            };

            let wasm = generator.generate(&js)?;

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
    file.write_all(PLUGIN_MODULE)?;
    Ok(())
}
