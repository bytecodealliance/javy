//! `javy profile`: profiles a Javy-generated module in a single step.
//!
//! Instrumenting, running, collecting and reporting all happen in one
//! command, rather than emitting an instrumented module for the user to
//! run themselves. The instrumentation produces two modules, the app
//! rewritten with probes and a state library backing those probes, and
//! the collected data lives in the state library's linear memory. The
//! profile only exists once the host links both modules, runs the app,
//! and asks the state library for its report. No off-the-shelf embedder
//! does that, so a standalone instrumented module would be an artifact
//! nobody could run usefully. Owning the whole pipeline also means the
//! report can be symbolized against the original module's bytecode
//! directly, without a separate analysis step.
//!
//! The tradeoff is that the program runs under this command's host
//! rather than its real embedder:
//!
//! - WASI is real: the program's stdin, stdout and stderr are the
//!   terminal's.
//! - Every other import is mocked with default values. Calls to host
//!   functions are not observed, and a program whose behavior depends on
//!   what they return may take different paths than it would in
//!   production. Host code cannot be optimized from a JS profile anyway,
//!   so this only matters insofar as it changes which JS runs.
//!
//! In the future we might open up a way to supply real implementations
//! of those imports, so programs that depend on their host can be
//! profiled faithfully.

use anyhow::Result;
use clap::Parser;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct ProfileOpts {
    #[arg(value_name = "INPUT", required = true)]
    /// Path of the statically linked Javy module to profile.
    pub input: PathBuf,

    #[arg(short, default_value = "profile.txt")]
    /// Output path of the profile.
    pub output: PathBuf,

    #[arg(long, value_name = "FUNCTION", default_value = javy_profiler::DEFAULT_INVOKE)]
    /// Exported function to call, as in `wasmtime run --invoke`. It must
    /// take no arguments and return no results.
    pub invoke: String,

    #[arg(long, value_name = "PATH")]
    /// Also write the raw binary report collected by the instrumentation.
    /// Intended for debugging the profiler itself.
    pub dump_trace: Option<PathBuf>,
}

/// Run the profiling command.
pub async fn run(opts: &ProfileOpts) -> Result<()> {
    let wasm = fs::read(&opts.input)?;
    let report = javy_profiler::profile(wasm, &opts.invoke).await?;
    if let Some(path) = &opts.dump_trace {
        fs::write(path, &report)?;
    }

    let mut records = javy_profiler::format::read(&report)?;

    // TODO: Symbolize records into JS function names. Until then, list
    // the raw records, hottest first.
    records.sort_by_key(|r| std::cmp::Reverse(r.count));
    let mut profile = format!("{:>10}  {:>6}  {:>12}\n", "func_addr", "target", "count");
    for r in &records {
        writeln!(
            profile,
            "{:#010x}  {:>6}  {:>12}",
            r.func_addr, r.target, r.count
        )?;
    }
    fs::write(&opts.output, profile)?;
    Ok(())
}
