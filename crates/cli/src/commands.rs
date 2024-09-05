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
#[derive(Clone, Debug)]
pub struct CodegenOptionGroup {
    /// Creates a smaller module that requires a dynamically linked QuickJS provider Wasm
    /// module to execute (see `emit-provider` command).
    pub dynamic: bool,
    /// The WIT options.
    pub wit: WitOptions,
    /// Enable source code compression, which generates smaller WebAssembly files at the cost of increased compile time. Defaults to enabled.
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
        /// Creates a smaller module that requires a dynamically linked QuickJS provider Wasm
        /// module to execute (see `emit-provider` command).
        Dynamic(bool),
        /// Optional path to WIT file describing exported functions.
        /// Only supports function exports with no arguments and no return values.
        Wit(PathBuf),
        /// Optional path to WIT file describing exported functions.
        /// Only supports function exports with no arguments and no return values.
        WitWorld(String),
        /// Enable source code compression, which generates smaller WebAssembly files at the cost of increased compile time. Defaults to enabled.
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

#[derive(Clone, Debug, PartialEq)]
pub struct JsOptionGroup {
    /// Whether to redirect console.log to stderr.
    pub redirect_stdout_to_stderr: bool,
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
        config
    }
}

impl From<Config> for JsOptionGroup {
    fn from(value: Config) -> Self {
        Self {
            redirect_stdout_to_stderr: value.contains(Config::REDIRECT_STDOUT_TO_STDERR),
        }
    }
}
