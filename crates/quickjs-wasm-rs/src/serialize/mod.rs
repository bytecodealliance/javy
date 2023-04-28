pub mod de;
pub mod err;
pub mod ser;

use super::js_binding::value::JSValueRef;

fn as_key<'a>(v: &'a JSValueRef) -> anyhow::Result<&'a str> {
    if v.is_str() {
        let v = v.as_str()?;
        Ok(v)
    } else {
        anyhow::bail!("map keys must be a string")
    }
}

#[cfg(test)]
mod tests {
    use super::de::Deserializer as ValueDeserializer;
    use super::ser::Serializer as ValueSerializer;
    use crate::js_binding::context::JSContextRef;
    use anyhow::Result;
    use quickcheck::quickcheck;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    quickcheck! {
        fn test_str(expected: String) -> Result<bool> {
            let actual = do_roundtrip::<_, String>(&expected);
            Ok(expected == actual)
        }

        fn test_u8(expected: u8) -> Result<bool> {
            let actual = do_roundtrip::<_, u8>(&expected);
            Ok(expected == actual)
        }

        fn test_u16(expected: u16) -> Result<bool> {
            let actual = do_roundtrip::<_, u16>(&expected);
            Ok(expected == actual)
        }

        fn test_f32(expected: f32) -> quickcheck::TestResult {
            if expected.is_nan() {
                return quickcheck::TestResult::discard();
            }

            let actual = do_roundtrip::<_, f32>(&expected);
            quickcheck::TestResult::from_bool(expected == actual)
        }

        fn test_i32(expected: i32) -> Result<bool> {
            let actual = do_roundtrip::<_, i32>(&expected);
            Ok(expected == actual)
        }

        // This test is not representative of what is happening in the real world. Since we are transcoding
        // from msgpack, only values greather than or equal to u32::MAX would be serialized as `BigInt`. Any other values would
        // be serialized as a `number`.
        //
        // See https://github.com/3Hren/msgpack-rust/blob/aa3c4a77b2b901fe73a555c615b92773b40905fc/rmp/src/encode/sint.rs#L170.
        //
        // This test works here since we are explicitly calling serialize_i64 and deserialize_i64.
        fn test_i64(expected: i64) -> Result<bool> {
            let actual = do_roundtrip::<_, i64>(&expected);
            Ok(expected == actual)
        }

        fn test_u32(expected: u32) -> Result<bool> {
            let actual = do_roundtrip::<_, u32>(&expected);
            Ok(expected == actual)
        }

        // This test is not representative of what is happening in the real world. Since we are transcoding
        // from msgpack, only values larger than i64::MAX would be serialized as BigInt. Any other values would
        // be serialized as a number.
        //
        // See https://github.com/3Hren/msgpack-rust/blob/aa3c4a77b2b901fe73a555c615b92773b40905fc/rmp/src/encode/sint.rs#L170.
        //
        // This test works here since we are explicitly calling serialize_u64 and deserialize_u64.
        fn test_u64(expected: u64) -> Result<bool> {
            let actual = do_roundtrip::<_, u64>(&expected);
            Ok(expected == actual)
        }

        fn test_bool(expected: bool) -> Result<bool> {
            let actual = do_roundtrip::<_, bool>(&expected);
            Ok(expected == actual)
        }
    }

    #[test]
    fn test_map() {
        let mut expected = BTreeMap::<String, String>::new();
        expected.insert("foo".to_string(), "bar".to_string());
        expected.insert("hello".to_string(), "world".to_string());

        let actual = do_roundtrip::<_, BTreeMap<String, String>>(&expected);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_struct_into_map() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObject {
            foo: String,
            bar: u32,
        }
        let expected = MyObject {
            foo: "hello".to_string(),
            bar: 1337,
        };

        let actual = do_roundtrip::<_, MyObject>(&expected);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nested_maps() {
        let mut expected = BTreeMap::<String, BTreeMap<String, String>>::new();
        let mut a = BTreeMap::new();
        a.insert("foo".to_string(), "bar".to_string());
        a.insert("hello".to_string(), "world".to_string());
        let mut b = BTreeMap::new();
        b.insert("toto".to_string(), "titi".to_string());
        expected.insert("aaa".to_string(), a);
        expected.insert("bbb".to_string(), b);

        let actual = do_roundtrip::<_, BTreeMap<String, BTreeMap<String, String>>>(&expected);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nested_structs_into_maps() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObjectB {
            toto: String,
            titi: i32,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObjectA {
            foo: String,
            bar: u32,
            b: MyObjectB,
        }
        let expected = MyObjectA {
            foo: "hello".to_string(),
            bar: 1337,
            b: MyObjectB {
                toto: "world".to_string(),
                titi: -42,
            },
        };

        let actual = do_roundtrip::<_, MyObjectA>(&expected);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sequence() {
        let expected = vec!["hello".to_string(), "world".to_string()];

        let actual = do_roundtrip::<_, Vec<String>>(&expected);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nested_sequences() {
        let mut expected = Vec::new();
        let a = vec!["foo".to_string(), "bar".to_string()];
        let b = vec!["toto".to_string(), "tata".to_string()];
        expected.push(a);
        expected.push(b);

        let actual = do_roundtrip::<_, Vec<Vec<String>>>(&expected);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sanity() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObject {
            a: u8,
            b: u16,
            c: u32,
            d: u64,
            e: i8,
            f: i16,
            g: i32,
            h: i64,
            i: f32,
            j: f64,
            k: String,
            l: bool,
            m: BTreeMap<String, u32>,
            n: Vec<u32>,
            o: BTreeMap<String, BTreeMap<String, u32>>,
            p: Vec<Vec<u32>>,
            bb: MyObjectB,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObjectB {
            a: u32,
            cc: MyObjectC,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct MyObjectC {
            a: Vec<u32>,
            b: BTreeMap<String, u32>,
        }

        let mut cc_b = BTreeMap::new();
        cc_b.insert("a".to_string(), 123);
        cc_b.insert("b".to_string(), 456);
        let cc = MyObjectC {
            a: vec![1337, 42],
            b: cc_b,
        };

        let bb = MyObjectB { a: 789, cc };

        let mut m = BTreeMap::new();
        m.insert("a".to_string(), 123);
        m.insert("b".to_string(), 456);
        m.insert("c".to_string(), 789);

        let mut oo = BTreeMap::new();
        oo.insert("e".to_string(), 123);

        let mut o = BTreeMap::new();
        o.insert("d".to_string(), oo);

        let expected = MyObject {
            a: u8::MAX,
            b: u16::MAX,
            c: u32::MAX,
            d: u64::MAX,
            e: i8::MAX,
            f: i16::MAX,
            g: i32::MAX,
            h: i64::MAX,
            i: f32::MAX,
            j: f64::MAX,
            k: "hello world".to_string(),
            l: true,
            m,
            n: vec![1, 2, 3, 4, 5],
            o,
            p: vec![vec![1, 2], vec![3, 4, 5]],
            bb,
        };

        let actual = do_roundtrip::<_, MyObject>(&expected);

        assert_eq!(expected, actual);
    }

    fn do_roundtrip<E, A>(expected: &E) -> A
    where
        E: Serialize,
        A: DeserializeOwned,
    {
        let context = JSContextRef::default();
        let mut serializer = ValueSerializer::from_context(&context).unwrap();
        expected.serialize(&mut serializer).unwrap();
        let mut deserializer = ValueDeserializer::from(serializer.value);
        let actual = A::deserialize(&mut deserializer).unwrap();
        actual
    }
}
