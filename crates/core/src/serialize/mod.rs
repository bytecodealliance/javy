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

    #[test]
    fn test_map() {
        let mut expected = BTreeMap::<String, String>::new();
        expected.insert("foo".to_string(), "bar".to_string());
        expected.insert("hello".to_string(), "world".to_string());

        let context = Context::default();
        let mut serializer = ValueSerializer::from_context(context);
        expected.serialize(&mut serializer).unwrap();
        let mut deserializer = ValueDeserializer::from(&context, serializer.value);
        let actual = BTreeMap::<String, String>::deserialize(&mut deserializer).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_struct_into_map() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObject {
            foo: String,
            bar: u32,
        }
        let expected = MyObject { foo: "hello".to_string(), bar: 1337 };

        let context = Context::default();
        let mut serializer = ValueSerializer::from_context(context);
        expected.serialize(&mut serializer).unwrap();
        let mut deserializer = ValueDeserializer::from(&context, serializer.value);
        let actual = MyObject::deserialize(&mut deserializer).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sequence() {
        let mut expected = Vec::new();
        expected.push("hello".to_string());
        expected.push("world".to_string());

        let actual = do_roundtrip::<_, Vec<String>>(&expected);

        assert_eq!(expected, actual);
    }

            fn do_roundtrip<E, A>(expected: E) -> A
        where
            E: Serialize,
            A: DeserializeOwned,
        {
            let context = Context::default();
            let mut serializer = ValueSerializer::from_context(context);
            expected.serialize(&mut serializer).unwrap();
            let mut deserializer = ValueDeserializer::from(&context, serializer.value);
            let actual = A::deserialize(&mut deserializer).unwrap();
            actual
        }
}
