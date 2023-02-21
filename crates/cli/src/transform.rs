use std::io::{Cursor, Write};

use anyhow::Result;
use brotli::enc::{self, BrotliEncoderParams};
use wasm_encoder::{CustomSection, Module};

pub const PRODUCERS_SECTION_NAME: &str = "producers";

pub fn add_producers_section(module: &mut Module) -> Result<()> {
    module.section(&CustomSection {
        name: PRODUCERS_SECTION_NAME,
        data: &producers_section_content()?,
    });
    Ok(())
}

pub fn add_source_code_section(module: &mut Module, source_code: &[u8]) -> Result<()> {
    let mut compressed_source_code: Vec<u8> = vec![];
    enc::BrotliCompress(
        &mut Cursor::new(source_code),
        &mut compressed_source_code,
        &BrotliEncoderParams {
            quality: 11,
            ..Default::default()
        },
    )?;

    module.section(&CustomSection {
        name: "javy_source",
        data: &compressed_source_code,
    });
    Ok(())
}

fn producers_section_content() -> Result<Vec<u8>> {
    // see https://github.com/WebAssembly/tool-conventions/blob/main/ProducersSection.md
    const FIELD_COUNT: u64 = 2;
    const LANGUAGE_FIELD_NAME: &str = "language";
    const LANGUAGE_FIELD_VALUE_COUNT: u64 = 1;
    const LANGUAGE: &str = "JavaScript";
    const LANGUAGE_VERSION: &str = "ES2020";
    const PROCESSED_BY_FIELD_NAME: &str = "processed-by";
    const PROCESSED_BY_FIELD_VALUE_COUNT: u64 = 1;
    const PROCESSED_BY: &str = "Javy";
    const PROCESSED_BY_VERSION: &str = env!("CARGO_PKG_VERSION");

    let mut producers_section = vec![];
    leb128::write::unsigned(&mut producers_section, FIELD_COUNT)?;
    write_wasm_vector(&mut producers_section, LANGUAGE_FIELD_NAME)?;
    leb128::write::unsigned(&mut producers_section, LANGUAGE_FIELD_VALUE_COUNT)?;
    write_wasm_vector(&mut producers_section, LANGUAGE)?;
    write_wasm_vector(&mut producers_section, LANGUAGE_VERSION)?;
    write_wasm_vector(&mut producers_section, PROCESSED_BY_FIELD_NAME)?;
    leb128::write::unsigned(&mut producers_section, PROCESSED_BY_FIELD_VALUE_COUNT)?;
    write_wasm_vector(&mut producers_section, PROCESSED_BY)?;
    write_wasm_vector(&mut producers_section, PROCESSED_BY_VERSION)?;

    Ok(producers_section)
}

fn write_wasm_vector(buffer: &mut Vec<u8>, s: &str) -> Result<()> {
    // see https://webassembly.github.io/spec/core/binary/conventions.html#binary-vec and
    // https://webassembly.github.io/spec/core/binary/values.html#binary-int for encoding details
    let bytes = s.as_bytes();
    leb128::write::unsigned(buffer, bytes.len().try_into()?)?;
    buffer.write_all(bytes)?;
    Ok(())
}
