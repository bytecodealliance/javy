use std::io::Cursor;

use anyhow::Result;
use brotli::enc::{self, BrotliEncoderParams};
use wasm_encoder::{CustomSection, Module};

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
