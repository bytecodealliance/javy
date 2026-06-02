use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    /// Instruments a Javy-generated module for profiling JS execution.
    #[command(arg_required_else_help = true)]
    Inject(ProfileInjectOpts),
}

#[derive(Debug, Parser)]
pub struct ProfileInjectOpts {
    #[arg(value_name = "INPUT", required = true)]
    /// Path of the WebAssembly module to instrument.
    pub input: PathBuf,

    #[arg(short, default_value = "profiled.wasm")]
    /// Output path of the instrumented WebAssembly output file.
    /// If no output is given, a `profiled.wasm` will be created
    /// in the same directory as the input program.
    pub output: PathBuf,
}

/// Run the profiling subcommand.
pub async fn run(cmd: &ProfileCommand) -> Result<()> {
    match cmd {
        ProfileCommand::Inject(opts) => inject(opts).await,
    }
}

async fn inject(opts: &ProfileInjectOpts) -> Result<()> {
    let wasm = fs::read(&opts.input)?;
    let output = javy_profiler::inject(wasm).await?;

    fs::write(&opts.output, &output.instrumented)?;

    let state_lib_path = opts
        .output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}.wasm", javy_profiler::LIBRARY_NAME));
    fs::write(&state_lib_path, &output.state_lib)?;
    Ok(())
}
