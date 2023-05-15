use anyhow::Result;
use quickjs_wasm_rs::{Deserializer, JSContextRef, JSValueRef, Serializer};

/// Transcodes a byte slice containing a MessagePack encoded payload into a [`JSValueRef`].
///
/// Arguments:
/// * `context` - A reference to the [`JSContextRef`] that will contain the
///   returned [`JSValueRef`].
/// * `bytes` - A byte slice containing a MessagePack encoded payload.
pub fn transcode_input<'a>(context: &'a JSContextRef, bytes: &[u8]) -> Result<JSValueRef<'a>> {
    let mut deserializer = rmp_serde::Deserializer::from_read_ref(bytes);
    let mut serializer = Serializer::from_context(context)?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(serializer.value)
}

/// Transcodes a [`JSValueRef`] into a MessagePack encoded byte vector.
pub fn transcode_output(val: JSValueRef) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(val);
    let mut serializer = rmp_serde::Serializer::new(&mut output);
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(output)
}
