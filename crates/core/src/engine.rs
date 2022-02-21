use anyhow::Result;
use std::io::{copy, stdin, stdout, Write};

pub fn load() -> Result<Vec<u8>> {
    let mut reader = stdin();
    let mut output: Vec<u8> = vec![];
    copy(&mut reader, &mut output)?;

    #[cfg(not(feature = "json-io"))]
    {
        Ok(output)
    }
    #[cfg(feature = "json-io")]
    {
        let output: serde_json::Value = serde_json::from_slice(&output)?;
        rmp_serde::to_vec(&output).map_err(Into::into)
    }
}

pub fn store(bytes: &[u8]) -> Result<()> {
    #[cfg(feature = "json-io")]
    let bytes = &{
        let value: serde_json::Value = rmp_serde::from_read_ref(bytes)?;
        serde_json::to_string(&value)?.into_bytes()
    };

    let mut handle = stdout();
    handle.write_all(bytes)?;

    Ok(())
}
