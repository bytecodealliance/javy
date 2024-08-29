use anyhow::{anyhow, bail};
use clap::{Parser, Subcommand};
use javy_config::Config;
use std::{path::PathBuf, str::FromStr};

use crate::codegen::WitOptions;

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

    #[arg(
        short = 'C',
        long = "codegen",
        long_help = "Available codegen options:
-C dynamic[=y|n] -- Creates a smaller module that requires a dynamically linked QuickJS provider Wasm module to execute (see `emit-provider` command).
-C wit=path -- Optional path to WIT file describing exported functions. Only supports function exports with no arguments and no return values.
-C wit-world=val -- Optional WIT world name for WIT file. Must be specified if WIT is file path is specified.
-C source-compression[=y|n] -- Enable source code compression, which generates smaller WebAssembly files at the cost of increased compile time. Defaults to enabled.
    "
    )]
    /// Codegen options.
    pub codegen: Vec<CodegenOption>,

    #[arg(
        short = 'J',
        long = "js",
        long_help = "Available JS runtime options:
-J redirect-stdout-to-stderr[=y|n] -- Redirects console.log to stderr.
        "
    )]
    /// JS runtime options.
    pub js_runtime: Vec<JsRuntimeOption>,
}

#[derive(Debug, Parser)]
pub struct EmitProviderCommandOpts {
    #[structopt(short, long)]
    /// Output path for the provider binary (default is stdout).
    pub out: Option<PathBuf>,
}

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

impl FromStr for CodegenOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '=');
        let key = parts.next().ok_or_else(|| anyhow!("Invalid codegen key"))?;
        let value = parts.next();
        let option = match key {
            "dynamic" => Self::Dynamic(match value {
                None => true,
                Some("y") => true,
                Some("n") => false,
                _ => bail!("Invalid value for dynamic"),
            }),
            "wit" => Self::Wit(PathBuf::from(
                value.ok_or_else(|| anyhow!("Must provide value for wit"))?,
            )),
            "wit-world" => Self::WitWorld(
                value
                    .ok_or_else(|| anyhow!("Must provide value for wit-world"))?
                    .to_string(),
            ),
            "source-compression" => Self::SourceCompression(match value {
                None => true,
                Some("y") => true,
                Some("n") => false,
                _ => bail!("Invalid value for source-compression"),
            }),
            _ => bail!("Invalid codegen key"),
        };
        Ok(option)
    }
}

impl TryFrom<Vec<CodegenOption>> for CodegenOptionGroup {
    type Error = anyhow::Error;

    fn try_from(value: Vec<CodegenOption>) -> Result<Self, Self::Error> {
        let mut options = Self::default();
        let mut wit = None;
        let mut wit_world = None;

        for option in value {
            match option {
                CodegenOption::Dynamic(enabled) => options.dynamic = enabled,
                CodegenOption::Wit(path) => wit = Some(path),
                CodegenOption::WitWorld(world) => wit_world = Some(world),
                CodegenOption::SourceCompression(enabled) => options.source_compression = enabled,
            }
        }

        options.wit = WitOptions::from_tuple((wit, wit_world))?;
        Ok(options)
    }
}

#[derive(Debug, PartialEq)]
pub struct JsRuntimeOptionGroup {
    /// Whether to redirect console.log to stderr.
    pub redirect_stdout_to_stderr: bool,
}

impl Default for JsRuntimeOptionGroup {
    fn default() -> Self {
        Config::default().into()
    }
}

#[derive(Clone, Debug)]
pub enum JsRuntimeOption {
    /// Whether to redirect console.log to stderr.
    RedirectStdoutToStderr(bool),
}

impl FromStr for JsRuntimeOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '=');
        let key = parts
            .next()
            .ok_or_else(|| anyhow!("Invalid JS runtime key"))?;
        let value = parts.next();
        let option = match key {
            "redirect-stdout-to-stderr" => Self::RedirectStdoutToStderr(match value {
                None => true,
                Some("y") => true,
                Some("n") => false,
                _ => bail!("Invalid value for redirect-stdout-to-stderr"),
            }),
            _ => bail!("Invalid JS runtime key"),
        };
        Ok(option)
    }
}

impl From<Vec<JsRuntimeOption>> for JsRuntimeOptionGroup {
    fn from(value: Vec<JsRuntimeOption>) -> Self {
        let mut group = Self::default();

        for option in value {
            match option {
                JsRuntimeOption::RedirectStdoutToStderr(enabled) => {
                    group.redirect_stdout_to_stderr = enabled;
                }
            }
        }

        group
    }
}

impl From<JsRuntimeOptionGroup> for Config {
    fn from(value: JsRuntimeOptionGroup) -> Self {
        let mut config = Self::default();
        config.set(
            Config::REDIRECT_STDOUT_TO_STDERR,
            value.redirect_stdout_to_stderr,
        );
        config
    }
}

impl From<Config> for JsRuntimeOptionGroup {
    fn from(value: Config) -> Self {
        Self {
            redirect_stdout_to_stderr: value.contains(Config::REDIRECT_STDOUT_TO_STDERR),
        }
    }
}
