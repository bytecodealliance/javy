use anyhow::Result;
use std::io::{copy, stdin, stdout, Write};

pub fn load() -> Result<Vec<u8>> {
    let mut reader = stdin();
    let mut output: Vec<u8> = vec![];

    copy(&mut reader, &mut output)?;

    Ok(output)
}

pub fn store(bytes: &[u8]) -> Result<()> {
    let mut handle = stdout();
    handle.write_all(bytes)?;
    handle.flush()?;

    Ok(())
}
