use crate::{option::OptionMeta, option_group, providers::Provider};
use anyhow::{anyhow, Result};
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
    /// This command will be deprecated in
    /// the next major release of the CLI (v4.0.0)
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
    /// Emits the provider binary that is required to run dynamically
    /// linked WebAssembly modules.
    EmitProvider(EmitProviderCommandOpts),
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
    /// Creates a smaller module that requires a dynamically linked QuickJS provider Wasm
    /// module to execute (see `emit-provider` command).
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

    #[arg(short = 'J', long = "javascript")]
    /// JavaScript runtime options.
    /// Use `-J help` for more details.
    pub js: Vec<RawGroupOption>,
    // pub js: Vec<GroupOption<JsOption>>,
}

#[derive(Debug, Parser)]
pub struct EmitProviderCommandOpts {
    #[structopt(short, long)]
    /// Output path for the provider binary (default is stdout).
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
}

impl Default for CodegenOptionGroup {
    fn default() -> Self {
        Self {
            dynamic: false,
            wit: WitOptions::default(),
            source_compression: true,
        }
    }
}

option_group! {
    #[derive(Clone, Debug)]
    pub enum CodegenOption {
        /// Creates a smaller module that requires a dynamically linked QuickJS
        /// provider Wasm module to execute (see `emit-provider` command).
        Dynamic(bool),
        /// Optional path to WIT file describing exported functions. Only
        /// supports function exports with no arguments and no return values.
        Wit(PathBuf),
        /// Optional path to WIT file describing exported functions. Only
        /// supports function exports with no arguments and no return values.
        WitWorld(String),
        /// Enable source code compression, which generates smaller WebAssembly
        /// files at the cost of increased compile time.
        SourceCompression(bool),
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
            }
        }

        options.wit = WitOptions::from_tuple((wit.cloned(), wit_world.cloned()))?;
        Ok(options)
    }
}

#[derive(Debug, Clone)]
pub struct RawGroupOptionParser;

#[derive(Debug, Clone)]
pub struct RawGroupOption((String, bool));

impl ValueParserFactory for RawGroupOption {
    type Parser = RawGroupOptionParser;

    fn value_parser() -> Self::Parser {
        RawGroupOptionParser
    }
}

impl TypedValueParser for RawGroupOptionParser {
    type Value = RawGroupOption;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        let val = StringValueParser::new().parse_ref(cmd, arg, value)?;

        if val == "help" {
            return Ok(RawGroupOption((val, false)));
        }

        let mut splits = val.splitn(2, '=');
        let key = splits.next().unwrap();
        let value = match splits.next() {
            Some("y") => true,
            Some("n") => false,
            None => true,
            _ => return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        };
        Ok(RawGroupOption((key.to_string(), value)))
    }
}

pub fn from_runtime_settings_to_config(
    provider: &Provider,
    runtime_settings: Vec<RawGroupOption>,
) -> Result<RuntimeConfig> {
    let supported_properties = provider.support_config()?;

    let mut supported_props = HashSet::new();
    for (key, _, _) in supported_properties.clone() {
        supported_props.insert(key);
    }

    let mut config = HashMap::new();
    for setting in runtime_settings {
        let (key, value) = setting.0;
        if key == "help" {
            fmt_help(
                "javascript",
                "J",
                &supported_properties
                    .clone()
                    .into_iter()
                    .map(|(name, _, doc)| OptionMeta {
                        name,
                        help: "[=y|n]".to_string(),
                        doc,
                    })
                    .collect::<Vec<_>>(),
            );
            std::process::exit(0);
        }
        if supported_props.contains(&key) {
            config.insert(key.to_string(), value);
        } else {
            Cli::command()
                .error(
                    ErrorKind::InvalidValue,
                    format!("Property {key} is not supported for runtime configuration"),
                )
                .exit();
        }
    }
    Ok(config)
}

pub type RuntimeConfig = HashMap<String, bool>;

// #[cfg(test)]
// mod tests {
//     use super::{CodegenOption, CodegenOptionGroup, GroupOption, JsOption, JsOptionGroup};
//     use anyhow::Result;
//     use javy_config::Config;

//     #[test]
//     fn js_group_conversion_between_vector_of_options_and_group() -> Result<()> {
//         let group: JsOptionGroup = vec![].into();

//         assert_eq!(group, JsOptionGroup::default());

//         let raw = vec![GroupOption(vec![JsOption::RedirectStdoutToStderr(false)])];
//         let group: JsOptionGroup = raw.into();
//         let expected = JsOptionGroup {
//             redirect_stdout_to_stderr: false,
//             ..Default::default()
//         };

//         assert_eq!(group, expected);

//         let raw = vec![GroupOption(vec![JsOption::JavyJson(false)])];
//         let group: JsOptionGroup = raw.into();
//         let expected = JsOptionGroup {
//             javy_json: false,
//             ..Default::default()
//         };
//         assert_eq!(group, expected);

//         let raw = vec![GroupOption(vec![JsOption::JavyStreamIo(false)])];
//         let group: JsOptionGroup = raw.into();
//         let expected = JsOptionGroup {
//             javy_stream_io: false,
//             ..Default::default()
//         };
//         assert_eq!(group, expected);

//         let raw = vec![GroupOption(vec![JsOption::SimdJsonBuiltins(false)])];
//         let group: JsOptionGroup = raw.into();

//         let expected = JsOptionGroup {
//             simd_json_builtins: false,
//             ..Default::default()
//         };
//         assert_eq!(group, expected);

//         let raw = vec![GroupOption(vec![JsOption::TextEncoding(false)])];
//         let group: JsOptionGroup = raw.into();

//         let expected = JsOptionGroup {
//             text_encoding: false,
//             ..Default::default()
//         };
//         assert_eq!(group, expected);

//         let raw = vec![GroupOption(vec![
//             JsOption::JavyStreamIo(false),
//             JsOption::JavyJson(false),
//             JsOption::RedirectStdoutToStderr(false),
//             JsOption::TextEncoding(false),
//             JsOption::SimdJsonBuiltins(false),
//         ])];
//         let group: JsOptionGroup = raw.into();
//         let expected = JsOptionGroup {
//             javy_stream_io: false,
//             javy_json: false,
//             redirect_stdout_to_stderr: false,
//             text_encoding: false,
//             simd_json_builtins: false,
//         };
//         assert_eq!(group, expected);

//         Ok(())
//     }

//     #[test]
//     fn codegen_group_conversion_between_vector_of_options_and_group() -> Result<()> {
//         let group: CodegenOptionGroup = vec![].try_into()?;
//         assert_eq!(group, CodegenOptionGroup::default());

//         let raw = vec![GroupOption(vec![CodegenOption::Dynamic(true)])];
//         let group: CodegenOptionGroup = raw.try_into()?;
//         let expected = CodegenOptionGroup {
//             dynamic: true,
//             ..Default::default()
//         };

//         assert_eq!(group, expected);

//         let raw = vec![GroupOption(vec![CodegenOption::SourceCompression(false)])];
//         let group: CodegenOptionGroup = raw.try_into()?;
//         let expected = CodegenOptionGroup {
//             source_compression: false,
//             ..Default::default()
//         };

//         assert_eq!(group, expected);

//         Ok(())
//     }

//     #[test]
//     fn js_conversion_between_group_and_config() -> Result<()> {
//         assert_eq!(JsOptionGroup::default(), Config::default().into());

//         let cfg: Config = JsOptionGroup::default().into();
//         assert_eq!(cfg, Config::default());
//         Ok(())
//     }
// }
