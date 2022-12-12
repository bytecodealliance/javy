mod js_module;
mod options;

use crate::options::Options;
use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::Path;
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

    add_custom_section(&opts.output, "javy_source".to_string(), js_text.as_bytes())?;

    Ok(())
}

fn add_custom_section<P: AsRef<Path>>(file: P, section: String, contents: &[u8]) -> Result<()> {
    use parity_wasm::elements::*;

    let mut compressed: Vec<u8> = vec![];
    brotli::enc::BrotliCompress(
        &mut std::io::Cursor::new(contents),
        &mut compressed,
        &brotli::enc::BrotliEncoderParams {
            quality: 11,
            ..Default::default()
        },
    )?;

    let mut module = parity_wasm::deserialize_file(&file)?;
    module
        .sections_mut()
        .push(Section::Custom(CustomSection::new(section, compressed)));
    parity_wasm::serialize_to_file(&file, module)?;

    Ok(())
}
