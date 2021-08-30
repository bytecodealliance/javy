pub mod de;
pub mod err;
pub mod ser;

#[cfg(test)]
mod tests {
    use super::de::Deserializer as ValueDeserializer;
    use super::ser::Serializer as ValueSerializer;
    use crate::js_binding::context::Context;
    use anyhow::Result;
    use quickcheck::quickcheck;
    use serde::{Deserialize, Serializer};

    quickcheck! {
        fn test_str_roundtrip(v: String) -> Result<bool> {
            let context = Context::default();
            let mut serializer = ValueSerializer::from_context(&context)?;
            serializer.serialize_str(v.as_str()).unwrap();

            let mut deserializer = ValueDeserializer::from_value(serializer.value)?;

            let result = String::deserialize(&mut deserializer).unwrap();
            Ok(v == result)
        }
    }
}
