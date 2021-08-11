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

    let engine: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));

    let working_dir = opts
        .working_dir
        .or_else(|| std::env::current_dir().ok())
        .expect("Failed to get current working directory");

    let optimized = opt::Optimizer::new(engine)
        .initialize(working_dir)?
        .strip()?
        .passes()?
        .wasm();

    std::fs::write(opts.output, &optimized)?;
    env::remove_var("JAVY_INPUT");
    Ok(())
}
