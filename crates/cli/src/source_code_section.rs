use anyhow::Result;

pub const SOURCE_CODE_SECTION_NAME: &str = "javy_source";

pub fn compress_source_code(source_code: &[u8]) -> Result<Vec<u8>> {
    let mut compressed_source_code: Vec<u8> = vec![];
    brotli::enc::BrotliCompress(
        &mut std::io::Cursor::new(source_code),
        &mut compressed_source_code,
        &brotli::enc::BrotliEncoderParams {
            quality: 11,
            ..Default::default()
        },
    )?;
    Ok(compressed_source_code)
}
