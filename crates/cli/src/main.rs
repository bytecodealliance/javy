mod js_module;
mod options;

use crate::options::Options;
use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use structopt::StructOpt;

fn main() -> Result<()> {
    let opts = Options::from_args();

    let mut contents = fs::File::open(&opts.input)
        .with_context(|| format!("Failed to open input file {}", opts.input.display()))?;

    let mut js_text = String::new();
    contents.read_to_string(&mut js_text).unwrap();
    let module = js_module::JsModule::new(&js_text);
    let js_wat = module.to_wat();

    let js_wasm_binary = wat::parse_str(js_wat)?;
    fs::write(&opts.output, &js_wasm_binary)?;
    Ok(())
}
