use anyhow::Result;
use std::io::Write;

use wasm_encoder::RawSection;
use wasmparser::{Parser, Payload::CustomSection};

pub const PRODUCERS_SECTION_NAME: &str = "producers";

pub fn update_producers_section(module: &[u8]) -> Result<Vec<u8>> {
    let mut output = wasm_encoder::Module::new();
    for payload in Parser::new(0).parse_all(module) {
        let payload = payload?;
        if let CustomSection(c) = &payload {
            if c.name() == PRODUCERS_SECTION_NAME {
                continue;
            }
        }
        if let Some((id, range)) = payload.as_section() {
            output.section(&RawSection {
                id,
                data: &module[range],
            });
        }
    }
    output.section(&wasm_encoder::CustomSection {
        name: PRODUCERS_SECTION_NAME,
        data: &producers_section_content()?,
    });
    Ok(output.as_slice().to_vec())
}

pub fn producers_section_content() -> Result<Vec<u8>> {
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
