use crate::js_binding::{context::Context, value::Value};
use crate::serialize::ser::Serializer;
use anyhow::Result;

pub fn prepare(context: &Context, bytes: &[u8]) -> Result<Value> {
    let mut deserializer = rmp_serde::Deserializer::from_read_ref(bytes);
    let mut serializer = Serializer::from_context(&context)?;
    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
    Ok(serializer.value)
}
