use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(about = "Initialize a Javy plugin")]
struct Args {
    #[arg(help = "Path to the uninitialized Javy plugin")]
    input: PathBuf,

    #[arg(help = "Output path for the initialized Javy plugin")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let wasm_bytes = fs::read(&args.input)?;
    let wasm_bytes = javy_plugin_processing::initialize_plugin(&wasm_bytes)?;
    fs::write(&args.output, wasm_bytes)?;
    Ok(())
}
