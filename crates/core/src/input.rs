use crate::value::*;

pub fn prepare(context: &crate::Context, bytes: &[u8]) -> Serializer {
    let mut deserializer = rmp_serde::Deserializer::from_read_ref(bytes);
    let mut serializer = Serializer::from_context(context.clone());
    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
    serializer
}

