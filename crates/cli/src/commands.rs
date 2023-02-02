use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "javy", about = "JavaScript to WebAssembly toolchain")]
pub enum Command {
    /// Compiles JavaScript to WebAssembly.
    Compile(CompileCommandOpts),
    /// Emits the provider binary that is required to run dynamically
    /// linked WebAssembly modules.
    EmitProvider(EmitProviderCommandOpts),
}

#[derive(Debug, StructOpt)]
pub struct CompileCommandOpts {
    #[structopt(parse(from_os_str))]
    /// Path of the JavaScript input file.
    pub input: PathBuf,

    #[structopt(short = "o", parse(from_os_str), default_value = "index.wasm")]
    /// Desired path of the WebAssembly output file.
    pub output: PathBuf,

    #[structopt(short = "d")]
    /// Creates a smaller module that requires a dynamically linked QuickJS provider Wasm
    /// module to execute (see `emit-provider` command).
    pub dynamic: bool,
}

#[derive(Debug, StructOpt)]
pub struct EmitProviderCommandOpts {
    #[structopt(long = "out", short = "o")]
    /// Output path for the provider binary (default is stdout).
    pub out: Option<PathBuf>,
}
