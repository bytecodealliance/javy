use crate::js_binding::{properties::Properties, value::Value};
use crate::serialize::err::{Error, Result};
use anyhow::anyhow;
use serde::de;
use serde::forward_to_deserialize_any;

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(anyhow!(msg.to_string()))
    }
}

pub struct Deserializer {
    value: Value,
}

impl Deserializer {
    pub fn from_value(value: Value) -> Result<Self> {
        Ok(Self { value })
    }

    fn is_null_or_undefined(&self) -> bool {
        self.value.is_null() | self.value.is_undefined()
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_repr_as_i32() {
            return visitor.visit_i32(self.value.inner() as i32);
        }

        if self.value.is_repr_as_f64() {
            let val = self.value.as_f64()?;
            return visitor.visit_f64(val);
        }

        if self.value.is_bool() {
            let val = self.value.as_bool()?;
            return visitor.visit_bool(val);
        }

        if self.is_null_or_undefined() {
            return visitor.visit_unit();
        }

        if self.value.is_str() {
            let val = self.value.as_str()?;
            return visitor.visit_str(&val);
        }

        if self.value.is_array() {
            let val = self.value.get_property("length")?;
            let length = val.inner() as u32;
            let seq = self.value.clone();
            let seq_access = SeqAccess {
                de: self,
                length,
                seq,
                i: 0,
            };
            return visitor.visit_seq(seq_access);
        }

        if self.value.is_object() {
            let properties = self.value.properties()?;
            let map_access = MapAccess {
                de: self,
                properties,
            };
            return visitor.visit_map(map_access);
        }

        Err(Error::Custom(anyhow!(
            "Couldn't deserialize value: {:?}",
            self.value
        )))
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.is_null_or_undefined() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct MapAccess<'a> {
    de: &'a mut Deserializer,
    properties: Properties,
}

impl<'a, 'de> de::MapAccess<'de> for MapAccess<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(key) = self.properties.next_key()? {
            self.de.value = key;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.value = self.properties.next_value()?;
        seed.deserialize(&mut *self.de)
    }
}

struct SeqAccess<'a> {
    de: &'a mut Deserializer,
    seq: Value,
    length: u32,
    i: u32,
}

impl<'a, 'de> de::SeqAccess<'de> for SeqAccess<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.i < self.length {
            self.de.value = self.seq.get_indexed_property(self.i as u32)?;
            self.i += 1;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Deserializer as ValueDeserializer;
    use crate::js_binding::context::Context;
    use anyhow::Result;
    use quickcheck::quickcheck;
    use serde::Deserialize;

    quickcheck! {
        fn test_i32(v: i32) -> Result<bool> {
            let context = Context::default();
            let val = context.value_from_i32(v)?;
            let mut deserializer = ValueDeserializer::from_value(val)?;

            let result = i32::deserialize(&mut deserializer)?;
            Ok(result == v)
        }

        fn test_bool(v: bool) -> Result<bool> {
            let context = Context::default();
            let val = context.value_from_bool(v)?;
            let mut deserializer = ValueDeserializer::from_value(val)?;

            let result = bool::deserialize(&mut deserializer)?;
            Ok(result == v)
        }

        fn test_str(v: String) -> Result<bool> {
            let context = Context::default();
            let val = context.value_from_str(&v)?;
            let mut deserializer = ValueDeserializer::from_value(val)?;

            let result = String::deserialize(&mut deserializer)?;
            Ok(result == v)
        }
    }

    #[test]
    fn test_null() -> Result<()> {
        let context = Context::default();
        let val = context.null_value()?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = <()>::deserialize(&mut deserializer)?;
        assert_eq!(result, ());
        Ok(())
    }

    #[test]
    fn test_undefined() -> Result<()> {
        let context = Context::default();
        let val = context.undefined_value()?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = <()>::deserialize(&mut deserializer)?;
        assert_eq!(result, ());
        Ok(())
    }

    #[test]
    fn test_nan() -> Result<()> {
        let context = Context::default();
        let val = context.value_from_f64(f64::NAN)?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = f64::deserialize(&mut deserializer)?;
        assert!(result.is_nan());
        Ok(())
    }

    #[test]
    fn test_infinity() -> Result<()> {
        let context = Context::default();
        let val = context.value_from_f64(f64::INFINITY)?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = f64::deserialize(&mut deserializer)?;
        assert!(result.is_infinite() && result.is_sign_positive());
        Ok(())
    }

    #[test]
    fn test_negative_infinity() -> Result<()> {
        let context = Context::default();
        let val = context.value_from_f64(f64::NEG_INFINITY)?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = f64::deserialize(&mut deserializer)?;
        assert!(result.is_infinite() && result.is_sign_negative());
        Ok(())
    }
}
