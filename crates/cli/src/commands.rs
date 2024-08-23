use anyhow::{anyhow, bail};
use clap::{Parser, Subcommand};
use std::{path::PathBuf, str::FromStr};

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

impl Command {
    /// Returns true if it is [`Command::Compile`].
    pub fn is_compile(&self) -> bool {
        matches!(self, Command::Compile(_))
    }
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
        long_help = "Available codegen options:
-C dynamic[=y|n] -- Creates a smaller module that requires a dynamically linked QuickJS provider Wasm module to execute (see `emit-provider` command).
-C wit=path -- Optional path to WIT file describing exported functions. Only supports function exports with no arguments and no return values.
-C wit-world=val -- Optional WIT world name for WIT file. Must be specified if WIT is file path is specified.
-C source-compression[=y|n] -- Enable source code compression, which generates smaller WebAssembly files at the cost of increased compile time. Defaults to enabled.
    "
    )]
    /// Codegen options.
    pub codegen: Vec<CodegenOption>,
}

#[derive(Debug, Parser)]
pub struct EmitProviderCommandOpts {
    #[structopt(short, long)]
    /// Output path for the provider binary (default is stdout).
    pub out: Option<PathBuf>,
}

#[derive(Clone, Debug, Parser)]
pub struct CodegenOptionGroup {
    /// Creates a smaller module that requires a dynamically linked QuickJS provider Wasm
    /// module to execute (see `emit-provider` command).
    pub dynamic: bool,
    /// Optional path to WIT file describing exported functions.
    /// Only supports function exports with no arguments and no return values.
    pub wit: Option<PathBuf>,
    /// Optional path to WIT file describing exported functions.
    /// Only supports function exports with no arguments and no return values.
    pub wit_world: Option<String>,
    /// Enable source code compression, which generates smaller WebAssembly files at the cost of increased compile time. Defaults to enabled.
    pub source_compression: bool,
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

impl From<Vec<CodegenOption>> for CodegenOptionGroup {
    fn from(value: Vec<CodegenOption>) -> Self {
        let mut dynamic = false;
        let mut wit = None;
        let mut wit_world = None;
        let mut source_compression = true;

        for option in value {
            match option {
                CodegenOption::Dynamic(enabled) => dynamic = enabled,
                CodegenOption::Wit(path) => wit = Some(path),
                CodegenOption::WitWorld(world) => wit_world = Some(world),
                CodegenOption::SourceCompression(enabled) => source_compression = enabled,
            }
        }

        Self {
            dynamic,
            wit,
            wit_world,
            source_compression,
        }
    }
}
