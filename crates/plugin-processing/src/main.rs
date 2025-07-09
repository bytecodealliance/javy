use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(about = "Extract a core Wasm module from a Wasm component")]
struct Args {
    #[arg(help = "Path to the Wasm component file")]
    input: PathBuf,

    #[arg(help = "Output path for the extracted module")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let component_bytes = fs::read(&args.input)?;
    let module_bytes = javy_plugin_processing::extract_core_module(&component_bytes)?;
    let module_bytes = javy_plugin_processing::optimize_module(&module_bytes)?;
    let module_bytes = javy_plugin_processing::preinitialize_module(&module_bytes)?;
    fs::write(&args.output, module_bytes)?;
    Ok(())
}
