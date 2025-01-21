use crate::{js_config::JsConfig, option::OptionMeta, option_group, plugins::Plugin};
use anyhow::{anyhow, bail, Result};
use clap::{
    builder::{StringValueParser, TypedValueParser, ValueParserFactory},
    error::ErrorKind,
    CommandFactory, Parser, Subcommand,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::codegen::WitOptions;
use crate::option::{
    fmt_help, GroupDescriptor, GroupOption, GroupOptionBuilder, GroupOptionParser, OptionValue,
};

#[derive(Debug, Parser)]
#[command(
    name = "javy",
    version,
    about = "JavaScript to WebAssembly toolchain",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compiles JavaScript to WebAssembly.
    ///
    /// NOTICE:
    ///
    /// This command is deprecated and will be removed.
    ///
    /// Refer to https://github.com/bytecodealliance/javy/issues/702 for
    /// details.
    ///
    /// Use the `build` command instead.
    #[command(arg_required_else_help = true)]
    Compile(CompileCommandOpts),
    /// Generates WebAssembly from a JavaScript source.
    #[command(arg_required_else_help = true)]
    Build(BuildCommandOpts),
    /// Emits the plugin binary that is required to run dynamically
    /// linked WebAssembly modules.
    EmitPlugin(EmitPluginCommandOpts),
    /// Initializes a plugin binary.
    #[command(arg_required_else_help = true)]
    InitPlugin(InitPluginCommandOpts),
}

#[derive(Debug, Parser)]
pub struct CompileCommandOpts {
    #[arg(value_name = "INPUT", required = true)]
    /// Path of the JavaScript input file.
    pub input: PathBuf,

    #[arg(short, default_value = "index.wasm")]
    /// Desired path of the WebAssembly output file.
    pub output: PathBuf,

    #[arg(short)]
    /// Creates a smaller module that requires a dynamically linked QuickJS
    /// plugin Wasm module to execute (see `emit-plugin` command).
    pub dynamic: bool,

    #[structopt(long)]
    /// Optional path to WIT file describing exported functions.
    /// Only supports function exports with no arguments and no return values.
    pub wit: Option<PathBuf>,

    #[arg(short = 'n')]
    /// Optional WIT world name for WIT file. Must be specified if WIT is file path is
    /// specified.
    pub wit_world: Option<String>,

    #[arg(long = "no-source-compression")]
    /// Disable source code compression, which reduces compile time at the expense of generating larger WebAssembly files.
    pub no_source_compression: bool,
}

const RUNTIME_CONFIG_ARG_SHORT: char = 'J';
const RUNTIME_CONFIG_ARG_LONG: &str = "javascript";

#[derive(Debug, Parser)]
pub struct BuildCommandOpts {
    #[arg(value_name = "INPUT", required = true)]
    /// Path of the JavaScript input file.
    pub input: PathBuf,

    #[arg(short, default_value = "index.wasm")]
    /// Desired path of the WebAssembly output file.
    pub output: PathBuf,

    #[arg(short = 'C', long = "codegen")]
    /// Code generation options.
    /// Use `-C help` for more details.
    pub codegen: Vec<GroupOption<CodegenOption>>,

    #[arg(short = RUNTIME_CONFIG_ARG_SHORT, long = RUNTIME_CONFIG_ARG_LONG)]
    /// JavaScript runtime options.
    /// Use `-J help` for more details.
    pub js: Vec<JsGroupValue>,
}

#[derive(Debug, Parser)]
pub struct EmitPluginCommandOpts {
    #[structopt(short, long)]
    /// Output path for the plugin binary (default is stdout).
    pub out: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct InitPluginCommandOpts {
    #[arg(value_name = "PLUGIN", required = true)]
    /// Path to the plugin to initialize.
    pub plugin: PathBuf,
    #[arg(short, long = "out")]
    /// Output path for the initialized plugin binary (default is stdout).
    pub out: Option<PathBuf>,
}

impl<T> ValueParserFactory for GroupOption<T>
where
    T: GroupDescriptor,
{
    type Parser = GroupOptionParser<T>;

    fn value_parser() -> Self::Parser {
        GroupOptionParser(std::marker::PhantomData)
    }
}

impl<T> TypedValueParser for GroupOptionParser<T>
where
    T: GroupDescriptor + GroupOptionBuilder,
{
    type Value = GroupOption<T>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let val = StringValueParser::new().parse_ref(cmd, arg, value)?;
        let arg = arg.expect("argument to be defined");
        let short = arg.get_short().expect("short version to be defined");
        let long = arg.get_long().expect("long version to be defined");

        if val == "help" {
            fmt_help(long, &short.to_string(), &T::options());
            std::process::exit(0);
        }

        let mut opts = vec![];

        for val in val.split(',') {
            opts.push(T::parse(val).map_err(|e| {
                clap::Error::raw(clap::error::ErrorKind::InvalidValue, format!("{}", e))
            })?)
        }

        Ok(GroupOption(opts))
    }
}

/// Code generation option group.
/// This group gets configured from the [`CodegenOption`] enum.
//
// NB: The documentation for each field is ommitted given that it's similar to
// the enum used to configured the group.
#[derive(Clone, Debug, PartialEq)]
pub struct CodegenOptionGroup {
    pub dynamic: bool,
    pub wit: WitOptions,
    pub source_compression: bool,
    pub plugin: Option<PathBuf>,
}

impl Default for CodegenOptionGroup {
    fn default() -> Self {
        Self {
            dynamic: false,
            wit: WitOptions::default(),
            source_compression: true,
            plugin: None,
        }
    }
}

option_group! {
    #[derive(Clone, Debug)]
    pub enum CodegenOption {
        /// Creates a smaller module that requires a dynamically linked QuickJS
        /// plugin Wasm module to execute (see `emit-plugin` command).
        Dynamic(bool),
        /// Optional path to WIT file describing exported functions. Only
        /// supports function exports with no arguments and no return values.
        Wit(PathBuf),
        /// Optional WIT world name for WIT file. Must be specified if WIT is
        /// file path is specified.
        WitWorld(String),
        /// Enable source code compression, which generates smaller WebAssembly
        /// files at the cost of increased compile time.
        SourceCompression(bool),
        /// Path to Javy plugin Wasm module. Optional for statically linked
        /// modules and required for dynamically linked modules. JavaScript
        /// config options are not supported when using this parameter.
        Plugin(PathBuf),
    }
}

impl TryFrom<Vec<GroupOption<CodegenOption>>> for CodegenOptionGroup {
    type Error = anyhow::Error;

    fn try_from(value: Vec<GroupOption<CodegenOption>>) -> Result<Self, Self::Error> {
        let mut options = Self::default();
        let mut wit = None;
        let mut wit_world = None;

        for option in value.iter().flat_map(|i| i.0.iter()) {
            match option {
                CodegenOption::Dynamic(enabled) => options.dynamic = *enabled,
                CodegenOption::Wit(path) => wit = Some(path),
                CodegenOption::WitWorld(world) => wit_world = Some(world),
                CodegenOption::SourceCompression(enabled) => options.source_compression = *enabled,
                CodegenOption::Plugin(path) => options.plugin = Some(path.clone()),
            }
        }

        options.wit = WitOptions::from_tuple((wit.cloned(), wit_world.cloned()))?;

        // We never want to assume the import namespace to use for a
        // dynamically linked module. If we do assume the import namespace, any
        // change to that assumed import namespace can result in new
        // dynamically linked modules not working on existing execution
        // environments because there will be unmet import errors when trying
        // to instantiate those modules. Since we can't assume the import
        // namespace, we must require a plugin so we can derive the import
        // namespace from the plugin.
        if options.dynamic && options.plugin.is_none() {
            bail!("Must specify plugin when using dynamic linking");
        }

        Ok(options)
    }
}

/// A runtime config group value.
#[derive(Debug, Clone)]
pub(super) enum JsGroupValue {
    Option(JsGroupOption),
    Help,
}

/// A runtime config group option.
#[derive(Debug, Clone)]
pub(super) struct JsGroupOption {
    /// The property name used for the option.
    name: String,
    /// Whether the config is enabled or not.
    enabled: bool,
}

#[derive(Debug, Clone)]
pub(super) struct JsGroupOptionParser;

impl ValueParserFactory for JsGroupValue {
    type Parser = JsGroupOptionParser;

    fn value_parser() -> Self::Parser {
        JsGroupOptionParser
    }
}

impl TypedValueParser for JsGroupOptionParser {
    type Value = JsGroupValue;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        let val = StringValueParser::new().parse_ref(cmd, arg, value)?;

        if val == "help" {
            return Ok(JsGroupValue::Help);
        }

        let mut splits = val.splitn(2, '=');
        let key = splits.next().unwrap();
        let value = match splits.next() {
            Some("y") => true,
            Some("n") => false,
            None => true,
            _ => return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        };
        Ok(JsGroupValue::Option(JsGroupOption {
            name: key.to_string(),
            enabled: value,
        }))
    }
}

impl JsConfig {
    /// Build a JS runtime config from valid runtime config values.
    pub(super) fn from_group_values(
        plugin: &Plugin,
        group_values: Vec<JsGroupValue>,
    ) -> Result<JsConfig> {
        let supported_properties = plugin.config_schema()?;

        let mut supported_names = HashSet::new();
        for property in &supported_properties {
            supported_names.insert(property.name.as_str());
        }

        let mut config = HashMap::new();
        for value in group_values {
            match value {
                JsGroupValue::Help => {
                    fmt_help(
                        RUNTIME_CONFIG_ARG_LONG,
                        &RUNTIME_CONFIG_ARG_SHORT.to_string(),
                        &supported_properties
                            .into_iter()
                            .map(|prop| OptionMeta {
                                name: prop.name,
                                help: "[=y|n]".to_string(),
                                doc: prop.doc,
                            })
                            .collect::<Vec<_>>(),
                    );
                    std::process::exit(0);
                }
                JsGroupValue::Option(JsGroupOption { name, enabled }) => {
                    if supported_names.contains(name.as_str()) {
                        config.insert(name, enabled);
                    } else {
                        Cli::command()
                            .error(
                                ErrorKind::InvalidValue,
                                format!(
                                    "Property {name} is not supported for runtime configuration",
                                ),
                            )
                            .exit();
                    }
                }
            }
        }
        Ok(JsConfig::from_hash(config))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        commands::{JsGroupOption, JsGroupValue},
        js_config::JsConfig,
        plugins::Plugin,
    };

    use super::{CodegenOption, CodegenOptionGroup, GroupOption};
    use anyhow::{Error, Result};

    #[test]
    fn js_config_from_config_values() -> Result<()> {
        let group = JsConfig::from_group_values(&Plugin::Default, vec![])?;
        assert_eq!(group.get("javy-stream-io"), None);
        assert_eq!(group.get("simd-json-builtins"), None);
        assert_eq!(group.get("text-encoding"), None);

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![JsGroupValue::Option(JsGroupOption {
                name: "javy-stream-io".to_string(),
                enabled: false,
            })],
        )?;
        assert_eq!(group.get("javy-stream-io"), Some(false));

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![JsGroupValue::Option(JsGroupOption {
                name: "javy-stream-io".to_string(),
                enabled: true,
            })],
        )?;
        assert_eq!(group.get("javy-stream-io"), Some(true));

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![JsGroupValue::Option(JsGroupOption {
                name: "simd-json-builtins".to_string(),
                enabled: false,
            })],
        )?;
        assert_eq!(group.get("simd-json-builtins"), Some(false));

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![JsGroupValue::Option(JsGroupOption {
                name: "simd-json-builtins".to_string(),
                enabled: true,
            })],
        )?;
        assert_eq!(group.get("simd-json-builtins"), Some(true));

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![JsGroupValue::Option(JsGroupOption {
                name: "text-encoding".to_string(),
                enabled: false,
            })],
        )?;
        assert_eq!(group.get("text-encoding"), Some(false));

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![JsGroupValue::Option(JsGroupOption {
                name: "text-encoding".to_string(),
                enabled: true,
            })],
        )?;
        assert_eq!(group.get("text-encoding"), Some(true));

        let group = JsConfig::from_group_values(
            &Plugin::Default,
            vec![
                JsGroupValue::Option(JsGroupOption {
                    name: "javy-stream-io".to_string(),
                    enabled: false,
                }),
                JsGroupValue::Option(JsGroupOption {
                    name: "simd-json-builtins".to_string(),
                    enabled: false,
                }),
                JsGroupValue::Option(JsGroupOption {
                    name: "text-encoding".to_string(),
                    enabled: false,
                }),
            ],
        )?;
        assert_eq!(group.get("javy-stream-io"), Some(false));
        assert_eq!(group.get("simd-json-builtins"), Some(false));
        assert_eq!(group.get("text-encoding"), Some(false));

        Ok(())
    }

    #[test]
    fn codegen_group_conversion_between_vector_of_options_and_group() -> Result<()> {
        let group: CodegenOptionGroup = vec![].try_into()?;
        assert_eq!(group, CodegenOptionGroup::default());

        let raw = vec![GroupOption(vec![
            CodegenOption::Dynamic(true),
            CodegenOption::Plugin(PathBuf::from("file.wasm")),
        ])];
        let group: CodegenOptionGroup = raw.try_into()?;
        let expected = CodegenOptionGroup {
            dynamic: true,
            plugin: Some(PathBuf::from("file.wasm")),
            ..Default::default()
        };

        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![CodegenOption::SourceCompression(false)])];
        let group: CodegenOptionGroup = raw.try_into()?;
        let expected = CodegenOptionGroup {
            source_compression: false,
            ..Default::default()
        };

        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![CodegenOption::Dynamic(true)])];
        let result: Result<CodegenOptionGroup, Error> = raw.try_into();
        assert_eq!(
            result.err().unwrap().to_string(),
            "Must specify plugin when using dynamic linking"
        );

        Ok(())
    }
}
