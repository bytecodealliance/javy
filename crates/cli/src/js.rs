use anyhow::{Context, Result};
use brotli::enc::{self, BrotliEncoderParams};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

use crate::bytecode;

pub struct JS {
    source_code: Vec<u8>,
}

impl JS {
    pub fn from_file(path: &Path) -> Result<JS> {
        let mut input_file = File::open(path)
            .with_context(|| format!("Failed to open input file {}", path.display()))?;
        let mut contents: Vec<u8> = vec![];
        input_file.read_to_end(&mut contents)?;
        Ok(JS {
            source_code: contents,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.source_code
    }

    pub fn compile(&self) -> Result<Vec<u8>> {
        bytecode::compile_source(&self.source_code)
    }

    pub fn compress(&self) -> Result<Vec<u8>> {
        let mut compressed_source_code: Vec<u8> = vec![];
        enc::BrotliCompress(
            &mut Cursor::new(&self.source_code),
            &mut compressed_source_code,
            &BrotliEncoderParams {
                quality: 11,
                ..Default::default()
            },
        )?;
        Ok(compressed_source_code)
    }
}
