mod options;
mod opt;

use std::env;
use anyhow::Result;
use structopt::StructOpt;
use crate::options::Options;


// TODO
// Use `join` here instead to ensure that this will work on Windows

fn main() -> Result<()> {
    let opts = Options::from_args();
    env::set_var("JAVY_INPUT", &opts.input);

    let engine: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));

    let optimized = opt::Optimizer::new(engine)
        .initialize()?
        .strip()?
        .passes()?
        .wasm();

    std::fs::write(opts.output, &optimized)?;
    env::remove_var("JAVY_INPUT");
    Ok(())
}
