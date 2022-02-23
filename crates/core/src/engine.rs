use anyhow::Result;
use std::io::{copy, stdin, stdout, Write};

pub fn load() -> Result<Vec<u8>> {
    let mut reader = stdin();
    let mut output: Vec<u8> = vec![];

    #[cfg(not(feature = "json-io"))]
    {
        copy(&mut reader, &mut output)?;
        Ok(output)
    }
    #[cfg(feature = "json-io")]
    {
        let mut input: Vec<u8> = vec![];
        copy(&mut reader, &mut input)?;
        let mut deserializer = serde_json::Deserializer::from_slice(&input);
        let mut serializer = rmp_serde::Serializer::new(&mut output);
        serde_transcode::transcode(&mut deserializer, &mut serializer)?;
        Ok(output)
    }
}
pub fn store(bytes: &[u8]) -> Result<()> {
    #[cfg(feature = "json-io")]
    let bytes = &{
        let mut output = Vec::new();
        let mut deserializer = rmp_serde::Deserializer::from_read_ref(bytes);
        let mut serializer = serde_json::Serializer::new(&mut output);
        serde_transcode::transcode(&mut deserializer, &mut serializer)?;
        output
    };

    let mut handle = stdout();
    handle.write_all(bytes)?;

    Ok(())
}
