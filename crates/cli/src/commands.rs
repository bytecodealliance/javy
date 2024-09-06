use crate::option_group;
use anyhow::{anyhow, Result};
use clap::{
    builder::{StringValueParser, TypedValueParser, ValueParserFactory},
    Parser, Subcommand,
};
use javy_config::Config;
use std::path::PathBuf;

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
    pub js: Vec<GroupOption<JsOption>>,
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

/// JavaScript option group.
/// This group gets configured from the [`JsOption`] enum.
//
// NB: The documentation for each field is ommitted given that it's similar to
// the enum used to configured the group.
#[derive(Clone, Debug, PartialEq)]
pub struct JsOptionGroup {
    pub redirect_stdout_to_stderr: bool,
    pub javy_json: bool,
    pub override_json_parse_and_stringify: bool,
    pub javy_stream_io: bool,
    pub text_encoding: bool,
}

impl Default for JsOptionGroup {
    fn default() -> Self {
        Config::default().into()
    }
}

option_group! {
    #[derive(Clone, Debug)]
    pub enum JsOption {
        /// Whether to redirect the output of console.log to standard error.
        RedirectStdoutToStderr(bool),
        /// Whether to enable the `Javy.JSON` builtins.
        JavyJson(bool),
        /// Whether to enable the `Javy.readSync` and `Javy.writeSync` builtins.
        JavyStreamIo(bool),
        /// Whether to override the `JSON.parse` and `JSON.stringify`
        /// implementations with an alternative, more performant, SIMD based
        /// implemetation.
        OverrideJsonParseAndStringify(bool),
        /// Whether to enable support for the `TextEncoder` and `TextDecoder`
        /// APIs.
        TextEncoding(bool),
    }
}

impl From<Vec<GroupOption<JsOption>>> for JsOptionGroup {
    fn from(value: Vec<GroupOption<JsOption>>) -> Self {
        let mut group = Self::default();

        for option in value.iter().flat_map(|e| e.0.iter()) {
            match option {
                JsOption::RedirectStdoutToStderr(enabled) => {
                    group.redirect_stdout_to_stderr = *enabled;
                }
                JsOption::JavyJson(enable) => group.javy_json = *enable,
                JsOption::OverrideJsonParseAndStringify(enable) => {
                    group.override_json_parse_and_stringify = *enable
                }
                JsOption::TextEncoding(enable) => group.text_encoding = *enable,
                JsOption::JavyStreamIo(enable) => group.javy_stream_io = *enable,
            }
        }

        group
    }
}

impl From<JsOptionGroup> for Config {
    fn from(value: JsOptionGroup) -> Self {
        let mut config = Self::default();
        config.set(
            Config::REDIRECT_STDOUT_TO_STDERR,
            value.redirect_stdout_to_stderr,
        );
        config.set(Config::JAVY_JSON, value.javy_json);
        config.set(
            Config::OVERRIDE_JSON_PARSE_AND_STRINGIFY,
            value.override_json_parse_and_stringify,
        );
        config.set(Config::JAVY_STREAM_IO, value.javy_stream_io);
        config.set(Config::TEXT_ENCODING, value.text_encoding);
        config
    }
}

impl From<Config> for JsOptionGroup {
    fn from(value: Config) -> Self {
        Self {
            redirect_stdout_to_stderr: value.contains(Config::REDIRECT_STDOUT_TO_STDERR),
            javy_json: value.contains(Config::JAVY_JSON),
            override_json_parse_and_stringify: value
                .contains(Config::OVERRIDE_JSON_PARSE_AND_STRINGIFY),
            javy_stream_io: value.contains(Config::JAVY_STREAM_IO),
            text_encoding: value.contains(Config::TEXT_ENCODING),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CodegenOption, CodegenOptionGroup, GroupOption, JsOption, JsOptionGroup};
    use anyhow::Result;
    use javy_config::Config;

    #[test]
    fn js_group_conversion_between_vector_of_options_and_group() -> Result<()> {
        let group: JsOptionGroup = vec![].into();

        assert_eq!(group, JsOptionGroup::default());

        let raw = vec![GroupOption(vec![JsOption::RedirectStdoutToStderr(false)])];
        let group: JsOptionGroup = raw.into();
        let mut expected = JsOptionGroup::default();

        expected.redirect_stdout_to_stderr = false;
        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![JsOption::JavyJson(false)])];
        let group: JsOptionGroup = raw.into();
        let mut expected = JsOptionGroup::default();

        expected.javy_json = false;
        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![JsOption::JavyStreamIo(false)])];
        let group: JsOptionGroup = raw.into();
        let mut expected = JsOptionGroup::default();

        expected.javy_stream_io = false;
        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![JsOption::OverrideJsonParseAndStringify(
            false,
        )])];
        let group: JsOptionGroup = raw.into();
        let mut expected = JsOptionGroup::default();

        expected.override_json_parse_and_stringify = false;
        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![JsOption::TextEncoding(false)])];
        let group: JsOptionGroup = raw.into();
        let mut expected = JsOptionGroup::default();

        expected.text_encoding = false;
        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![
            JsOption::JavyStreamIo(false),
            JsOption::JavyJson(false),
            JsOption::RedirectStdoutToStderr(false),
            JsOption::TextEncoding(false),
            JsOption::OverrideJsonParseAndStringify(false),
        ])];
        let group: JsOptionGroup = raw.into();
        let mut expected = JsOptionGroup::default();

        expected.text_encoding = false;
        expected.override_json_parse_and_stringify = false;
        expected.javy_json = false;
        expected.javy_stream_io = false;
        expected.redirect_stdout_to_stderr = false;

        assert_eq!(group, expected);

        Ok(())
    }

    #[test]
    fn codegen_group_conversion_between_vector_of_options_and_group() -> Result<()> {
        let group: CodegenOptionGroup = vec![].try_into()?;
        assert_eq!(group, CodegenOptionGroup::default());

        let raw = vec![GroupOption(vec![CodegenOption::Dynamic(true)])];
        let group: CodegenOptionGroup = raw.try_into()?;
        let mut expected = CodegenOptionGroup::default();
        expected.dynamic = true;

        assert_eq!(group, expected);

        let raw = vec![GroupOption(vec![CodegenOption::SourceCompression(false)])];
        let group: CodegenOptionGroup = raw.try_into()?;
        let mut expected = CodegenOptionGroup::default();
        expected.source_compression = false;

        assert_eq!(group, expected);

        Ok(())
    }

    #[test]
    fn js_conversion_between_group_and_config() -> Result<()> {
        assert_eq!(JsOptionGroup::default(), Config::default().into());

        let cfg: Config = JsOptionGroup::default().into();
        assert_eq!(cfg, Config::default());
        Ok(())
    }
}
