use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "javy", about = "JavaScript to WebAssembly toolchain")]
pub struct Options {
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short = "o", parse(from_os_str), default_value = "index.wasm")]
    pub output: PathBuf,

    #[structopt(short = "d")]
    /// Creates a smaller module that requires a dynamically linked QuickJS provider Wasm
    /// module to execute the JavaScript
    pub dynamic: bool,
}
