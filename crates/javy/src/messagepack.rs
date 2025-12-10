use crate::quickjs::{Ctx, Value};
use anyhow::Result;
use rquickjs_serde::{de::Deserializer, ser::Serializer};

/// Transcodes a byte slice containing a MessagePack encoded payload into a [`JSValueRef`].
///
/// Arguments:
/// * `context` - A reference to the [`JSContextRef`] that will contain the
///   returned [`JSValueRef`].
/// * `bytes` - A byte slice containing a MessagePack encoded payload.
pub fn transcode_input<'js>(context: Ctx<'js>, bytes: &[u8]) -> Result<Value<'js>> {
    let mut deserializer = rmp_serde::Deserializer::from_read_ref(bytes);
    let mut serializer = Serializer::from_context(context.clone())?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(serializer.value)
}

/// Transcodes a [`JSValueRef`] into a MessagePack encoded byte vector.
pub fn transcode_output(val: Value<'_>) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(val).with_strict();
    let mut serializer = rmp_serde::Serializer::new(&mut output);
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(output)
}
