use quickjs_sys as q;
use crate::serialization::Deserializer;

pub fn prepare(context: &crate::Context, val: q::JSValue) -> Vec<u8> {
    let mut output = Vec::new();
    let mut deserializer = Deserializer::from(&context, val);
    let mut serializer = rmp_serde::Serializer::new(&mut output);

    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();

    output
}

