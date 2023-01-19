mod bytecode;
mod module_generator;
mod opt;
mod options;
mod source_code_section;

use crate::options::Options;
use anyhow::{bail, Context, Result};
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
use std::{fs, process::Command};
use structopt::StructOpt;

fn main() -> Result<()> {
    let opts = Options::from_args();

    if opts.dynamic {
        create_dynamically_linked_module(opts)?;
    } else {
        create_statically_linked_module(opts)?;
    }

    Ok(())
}

fn create_statically_linked_module(opts: Options) -> Result<()> {
    let wizen = env::var("JAVY_WIZEN");

    if wizen.eq(&Ok("1".into())) {
        let wasm: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
        opt::Optimizer::new(wasm)
            .optimize(true)
            .write_optimized_wasm(opts.output)?;

        env::remove_var("JAVY_WIZEN");

        return Ok(());
    }

    let contents = read_input_file(&opts.input)?;

    let self_cmd = env::args().next().unwrap();

    {
        env::set_var("JAVY_WIZEN", "1");
        let mut command = Command::new(self_cmd)
            .arg(&opts.input)
            .arg("-o")
            .arg(&opts.output)
            .stdin(Stdio::piped())
            .spawn()?;
        command.stdin.take().unwrap().write_all(&contents)?;
        let status = command.wait()?;
        if !status.success() {
            bail!("Couldn't create wasm from input");
        }
    }

    add_custom_section(
        &opts.output,
        source_code_section::SOURCE_CODE_SECTION_NAME.to_string(),
        contents,
    )?;

    Ok(())
}

fn add_custom_section<P: AsRef<Path>>(file: P, section: String, contents: Vec<u8>) -> Result<()> {
    use parity_wasm::elements::*;

    let compressed = source_code_section::compress_source_code(&contents)?;

    let mut module = parity_wasm::deserialize_file(&file)?;
    module
        .sections_mut()
        .push(Section::Custom(CustomSection::new(section, compressed)));
    parity_wasm::serialize_to_file(&file, module)?;

    Ok(())
}

fn create_dynamically_linked_module(opts: Options) -> Result<()> {
    let js_source_code = read_input_file(&opts.input)?;
    let quickjs_bytecode = bytecode::compile_source(&js_source_code)?;
    let wasm_module = module_generator::generate_module(quickjs_bytecode, &js_source_code)?;
    let mut output_file = fs::File::create(&opts.output)?;
    output_file.write_all(&wasm_module)?;
    Ok(())
}

fn read_input_file(path: &Path) -> Result<Vec<u8>> {
    let mut input_file = fs::File::open(path)
        .with_context(|| format!("Failed to open input file {}", path.display()))?;
    let mut contents: Vec<u8> = vec![];
    input_file.read_to_end(&mut contents)?;
    Ok(contents)
}
