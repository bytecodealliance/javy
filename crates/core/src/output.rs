use crate::js_binding::value::Value;
use crate::serialize::de::Deserializer;
use anyhow::Result;

pub fn prepare(val: Value) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(val);
    let mut serializer = rmp_serde::Serializer::new(&mut output);

    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();

    Ok(output)
}
