mod opt;
mod options;
mod prebuilt;

use crate::options::Options;
use anyhow::Result;
use std::env;
use structopt::StructOpt;

// TODO
// Use `join` here instead to ensure that this will work on Windows

fn main() -> Result<()> {
    let opts = Options::from_args();
    env::set_var("JAVY_INPUT", &opts.input);

    let wasm: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));

    opt::Optimizer::new(wasm, opts.input.clone())
        .strip(true)
        .optimize(true)
        .write_optimized_wasm(opts.output)?;

    env::remove_var("JAVY_INPUT");
    Ok(())
}
