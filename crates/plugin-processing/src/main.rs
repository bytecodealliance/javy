use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use javy_plugin_processing::PluginConfig;

#[derive(Parser)]
#[command(about = "Initialize a Javy plugin")]
struct Args {
    #[arg(help = "Path to the uninitialized Javy plugin")]
    input: PathBuf,

    #[arg(help = "Output path for the initialized Javy plugin")]
    output: PathBuf,

    #[arg(
        long,
        help = "Produce deterministic output by using fixed clocks and constant zero-filled RNG. Security note: both secure_random and insecure_random become non-secure."
    )]
    deterministic: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = PluginConfig {
        deterministic: args.deterministic,
    };
    let wasm_bytes = fs::read(&args.input)?;
    let wasm_bytes =
        javy_plugin_processing::initialize_plugin_with_config(&wasm_bytes, &config).await?;
    fs::write(&args.output, wasm_bytes)?;
    Ok(())
}
