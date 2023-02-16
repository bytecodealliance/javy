mod bytecode;
mod commands;
mod module_generator;
mod opt;
mod producers_section;
mod source_code_section;

use crate::commands::{Command, CompileCommandOpts, EmitProviderCommandOpts};
use anyhow::{bail, Context, Result};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
use std::{fs, process::Command as OsCommand};
use structopt::StructOpt;

fn main() -> Result<()> {
    let cmd = Command::from_args();

    match &cmd {
        Command::EmitProvider(opts) => emit_provider(opts),
        Command::Compile(opts) => {
            if opts.dynamic {
                create_dynamically_linked_module(opts)
            } else {
                create_statically_linked_module(opts)
            }
        }
    }
}

fn emit_provider(opts: &EmitProviderCommandOpts) -> Result<()> {
    let mut file: Box<dyn Write> = match opts.out.as_ref() {
        Some(path) => Box::new(File::create(path)?),
        _ => Box::new(std::io::stdout()),
    };
    file.write_all(bytecode::QUICKJS_PROVIDER_MODULE)?;
    Ok(())
}

fn create_statically_linked_module(opts: &CompileCommandOpts) -> Result<()> {
    let wizen = env::var("JAVY_WIZEN");

    if wizen.eq(&Ok("1".into())) {
        let wasm: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
        opt::Optimizer::new(wasm)
            .optimize(true)
            .write_optimized_wasm(&opts.output)?;

        env::remove_var("JAVY_WIZEN");

        return Ok(());
    }

    let contents = read_input_file(&opts.input)?;

    let self_cmd = env::args().next().unwrap();

    {
        env::set_var("JAVY_WIZEN", "1");
        let mut command = OsCommand::new(self_cmd)
            .arg("compile")
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

    update_producers_section(&opts.output)?;

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

fn update_producers_section(file: &Path) -> Result<()> {
    let bytes = fs::read(file)?;
    let module_bytes = producers_section::update_producers_section(&bytes)?;
    fs::write(file, module_bytes)?;
    Ok(())
}

fn create_dynamically_linked_module(opts: &CompileCommandOpts) -> Result<()> {
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
