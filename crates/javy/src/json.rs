use crate::quickjs::{Ctx, Value};
use crate::serde::{de::Deserializer, ser::Serializer};
use anyhow::Result;

/// Transcodes a byte slice containing a JSON encoded payload into a [Value].
pub fn transcode_input<'js>(context: Ctx<'js>, bytes: &[u8]) -> Result<Value<'js>> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let mut serializer = Serializer::from_context(context)?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(serializer.value)
}

/// Transcodes a [Value] into a slice of JSON bytes.
pub fn transcode_output(val: Value<'_>) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(val);
    let mut serializer = serde_json::Serializer::new(&mut output);
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(output)
}
