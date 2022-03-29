use crate::serialize::de::Deserializer;
use crate::serialize::ser::Serializer;
use crate::{Context, Value};
use anyhow::Result;

pub fn transcode_input(context: &Context, bytes: &[u8]) -> Result<Value> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let mut serializer = Serializer::from_context(context)?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(serializer.value)
}

pub fn transcode_output(val: Value) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(val);
    let mut serializer = serde_json::Serializer::new(&mut output);
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(output)
}
