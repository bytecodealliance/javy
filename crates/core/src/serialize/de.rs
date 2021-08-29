use crate::js_binding::{own_properties::OwnProperties, value::Value};
use crate::serialize::err::{Error, Result};
use anyhow::anyhow;
use serde::de;

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(anyhow!(msg.to_string()))
    }
}

pub struct Deserializer {
    value: Value,
    own_properties: Option<OwnProperties>,
    length: isize,
    offset: isize,
}

impl Deserializer {
    pub fn from_value(value: Value) -> Result<Self> {
        Ok(Self {
            value,
            own_properties: None,
            length: 0,
            offset: 0,
        })
    }

    pub fn derive_own_properties(&mut self) -> Result<()> {
        self.own_properties = Some(self.value.own_properties()?);
        Ok(())
    }

    pub fn derive_seq_metadata(&mut self) -> Result<()> {
        let val = self.value.get_property("length")?;
        self.length = val.inner() as isize;
        self.offset = 0 as isize;
        Ok(())
    }

    pub fn seed_next_element(&mut self) -> Result<bool> {
        if self.offset >= self.length {
            self.offset = 0;
            self.length = 0;
            Ok(false)
        } else {
            self.value = self.value.get_indexed_property(self.offset as u32)?;
            Ok(true)
        }
    }

    pub fn seed_next_key(&mut self) -> Result<bool> {
        if let Some(props) = &mut self.own_properties {
            if let Some(k) = props.next_key()? {
                self.value = k;
                return Ok(true);
            } else {
                self.own_properties = None;
                return Ok(false);
            }
        }

        Ok(false)
    }

    pub fn seed_next_value(&mut self) -> Result<bool> {
        if let Some(props) = &self.own_properties {
            self.value = props.next_value()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_repr_as_i32() {
            return self.deserialize_i32(visitor);
        }

        if self.value.is_repr_as_f64() {
            return self.deserialize_f64(visitor);
        }

        if self.value.is_bool() {
            return self.deserialize_bool(visitor);
        }

        if self.value.is_null() || self.value.is_undefined() {
            return self.deserialize_unit(visitor);
        }

        if self.value.is_str() {
            return self.deserialize_str(visitor);
        }

        if self.value.is_array() {
            return self.deserialize_seq(visitor);
        }

        if self.value.is_object() {
            return self.deserialize_map(visitor);
        }

        Err(Error::Custom(anyhow!(
            "Couldn't deserialize value: {:?}",
            self.value
        )))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.value.inner() as i32)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.value.as_f64()?;
        visitor.visit_f64(val)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.value.as_bool()?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let string = self.value.as_str()?;
        visitor.visit_str(&string)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.derive_seq_metadata()?;
        visitor.visit_seq(&mut *self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.derive_own_properties()?;
        visitor.visit_map(&mut *self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
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

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> de::MapAccess<'de> for Deserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.seed_next_key()? {
            seed.deserialize(&mut *self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.seed_next_value()?;
        seed.deserialize(&mut *self)
    }
}

impl<'de> de::SeqAccess<'de> for Deserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.seed_next_element()? {
            seed.deserialize(&mut *self).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Deserializer as ValueDeserializer;
    use crate::js_binding::{context::Context, value::Value};
    use anyhow::Result;
    use quickcheck::quickcheck;
    use serde::Deserialize;

    quickcheck! {
        fn test_i32(v: i32) -> Result<bool> {
            let context = Context::default();
            let val = Value::from_i32(context.inner(), v)?;
            let mut deserializer = ValueDeserializer::from_value(val)?;

            let result = i32::deserialize(&mut deserializer)?;
            Ok(result == v)
        }

        fn test_bool(v: bool) -> Result<bool> {
            let context = Context::default();
            let val = Value::from_bool(context.inner(), v)?;
            let mut deserializer = ValueDeserializer::from_value(val)?;

            let result = bool::deserialize(&mut deserializer)?;
            Ok(result == v)
        }

        fn test_str(v: String) -> Result<bool> {
            let context = Context::default();
            let val = Value::from_str(context.inner(), &v)?;
            let mut deserializer = ValueDeserializer::from_value(val)?;

            let result = String::deserialize(&mut deserializer)?;
            Ok(result == v)
        }
    }

    #[test]
    fn test_null() -> Result<()> {
        let context = Context::default();
        let val = Value::null(context.inner())?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = <()>::deserialize(&mut deserializer)?;
        assert_eq!(result, ());
        Ok(())
    }

    #[test]
    fn test_undefined() -> Result<()> {
        let context = Context::default();
        let val = Value::undefined(context.inner())?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = <()>::deserialize(&mut deserializer)?;
        assert_eq!(result, ());
        Ok(())
    }

    #[test]
    fn test_nan() -> Result<()> {
        let context = Context::default();
        let val = Value::from_f64(context.inner(), f64::NAN)?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = f64::deserialize(&mut deserializer)?;
        assert!(result.is_nan());
        Ok(())
    }

    #[test]
    fn test_infinity() -> Result<()> {
        let context = Context::default();
        let val = Value::from_f64(context.inner(), f64::INFINITY)?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = f64::deserialize(&mut deserializer)?;
        assert!(result.is_infinite() && result.is_sign_positive());
        Ok(())
    }

    #[test]
    fn test_negative_infinity() -> Result<()> {
        let context = Context::default();
        let val = Value::from_f64(context.inner(), f64::NEG_INFINITY)?;
        let mut deserializer = ValueDeserializer::from_value(val)?;

        let result = f64::deserialize(&mut deserializer)?;
        assert!(result.is_infinite() && result.is_sign_negative());
        Ok(())
    }
}
