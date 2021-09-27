mod opt;
mod options;

use crate::options::Options;
use anyhow::Result;
use std::env;
use structopt::StructOpt;

// TODO
// Use `join` here instead to ensure that this will work on Windows

fn main() -> Result<()> {
    let opts = Options::from_args();
    let canonical = std::fs::canonicalize(&opts.input)?;
    env::set_var("JAVY_INPUT", &canonical);

    let wasm: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));

    opt::Optimizer::new(wasm, canonical)
        .optimize(true)
        .write_optimized_wasm(opts.output)?;

    env::remove_var("JAVY_INPUT");
    Ok(())
}
