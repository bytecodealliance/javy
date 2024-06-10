use crate::quickjs::{Ctx, Value};
use crate::serde::{de::Deserializer, ser::Serializer};
use anyhow::Result;

/// Transcodes a byte slice containing a JSON encoded payload into a [Value].
pub fn parse<'js>(context: Ctx<'js>, bytes: &mut [u8]) -> Result<Value<'js>> {
    let mut deserializer = simd_json::Deserializer::from_slice(bytes)?;
    let mut serializer = Serializer::from_context(context.clone())?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;

    Ok(serializer.value)
}

/// Transcodes a [Value] into a slice of JSON bytes.
pub fn stringify(val: Value<'_>) -> Result<Vec<u8>> {
    let mut output: Vec<u8> = Vec::new();
    let mut deserializer = Deserializer::from(val);
    let mut serializer = serde_json::Serializer::new(&mut output);
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(output)
}
