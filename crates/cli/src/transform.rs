use std::{borrow::Cow, io::Cursor};

use anyhow::Result;
use brotli::enc::{self, BrotliEncoderParams};
use walrus::{CustomSection, IdsToIndices, ModuleConfig, ModuleProducers};

#[derive(Debug)]
pub struct SourceCodeSection {
    compressed_source_code: Vec<u8>,
}

impl SourceCodeSection {
    pub fn new(source_code: &[u8]) -> Result<SourceCodeSection> {
        let mut compressed_source_code: Vec<u8> = vec![];
        enc::BrotliCompress(
            &mut Cursor::new(source_code),
            &mut compressed_source_code,
            &BrotliEncoderParams {
                quality: 11,
                ..Default::default()
            },
        )?;
        Ok(SourceCodeSection {
            compressed_source_code,
        })
    }
}

impl CustomSection for SourceCodeSection {
    fn name(&self) -> &str {
        "javy_source"
    }

    fn data(&self, _ids_to_indices: &IdsToIndices) -> Cow<[u8]> {
        (&self.compressed_source_code).into()
    }
}

pub fn module_config() -> ModuleConfig {
    let mut config = ModuleConfig::new();
    config.generate_name_section(false);
    config
}

pub fn add_producers_section(producers: &mut ModuleProducers) {
    producers.clear(); // removes Walrus and Rust
    producers.add_language("JavaScript", "ES2020");
    producers.add_processed_by("Javy", env!("CARGO_PKG_VERSION"));
}
