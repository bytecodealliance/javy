use crate::quickjs::{object::ObjectIter, Array, Filter, Value};
use crate::serde::err::{Error, Result};
use crate::serde::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER};
use crate::{from_js_error, to_string_lossy};
use anyhow::anyhow;
use serde::de::{self, Error as SerError};
use serde::forward_to_deserialize_any;

use super::as_key;

impl SerError for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(anyhow!(msg.to_string()))
    }
}

/// `Deserializer` is a deserializer for [Value] values, implementing the `serde::Deserializer` trait.
///
/// This struct is responsible for converting [Value], into Rust types using the Serde deserialization framework.
///
/// # Example
///
/// ```
/// // Assuming you have a [Value] instance named value containing an i32.
/// let mut deserializer = Deserializer::from(value);
///
/// // Use deserializer to deserialize the JavaScript value into a Rust type.
/// let number: i32 = serde::Deserialize::deserialize(deserializer)?;
/// ```
pub struct Deserializer<'js> {
    value: Value<'js>,
    map_key: bool,
    current_kv: Option<(Value<'js>, Value<'js>)>,
}

impl<'de> From<Value<'de>> for Deserializer<'de> {
    fn from(value: Value<'de>) -> Self {
        Self {
            value,
            map_key: false,
            current_kv: None,
        }
    }
}
impl<'js> Deserializer<'js> {
    fn deserialize_number<'de, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_int() {
            return visitor.visit_i32(
                self.value
                    .as_int()
                    .ok_or_else(|| anyhow!("Failed to convert value to i32"))?,
            );
        }

        if self.value.is_float() {
            let f64_representation = self
                .value
                .as_float()
                .ok_or_else(|| anyhow!("Failed to convert value to f64"))?;
            let is_positive = f64_representation.is_sign_positive();
            let safe_integer_range = (MIN_SAFE_INTEGER as f64)..=(MAX_SAFE_INTEGER as f64);
            let whole = f64_representation.fract() == 0.0;

            if whole && is_positive && f64_representation <= u32::MAX as f64 {
                return visitor.visit_u32(f64_representation as u32);
            }

            if whole && safe_integer_range.contains(&f64_representation) {
                let x = f64_representation as i64;
                return visitor.visit_i64(x);
            }

            return visitor.visit_f64(f64_representation);
        }
        unreachable!()
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.value.is_number() {
            return self.deserialize_number(visitor);
        }

        if self.value.is_bool() {
            let val = self.value.as_bool().unwrap();
            return visitor.visit_bool(val);
        }

        if self.value.is_null() || self.value.is_undefined() {
            return visitor.visit_unit();
        }

        if self.value.is_string() {
            if self.map_key {
                self.map_key = false;
                let key = as_key(&self.value)?;
                return visitor.visit_str(&key);
            } else {
                let val = self
                    .value
                    .as_string()
                    .map(|s| {
                        s.to_string()
                            .unwrap_or_else(|e| to_string_lossy(self.value.ctx(), s, e))
                    })
                    .unwrap();
                return visitor.visit_str(&val);
            }
        }

        if self.value.is_array() {
            let arr = self.value.as_array().unwrap().clone();
            let length = arr.len();
            let seq_access = SeqAccess {
                de: self,
                length,
                seq: arr,
                index: 0,
            };
            return visitor.visit_seq(seq_access);
        }

        if self.value.is_object() {
            let filter = Filter::new().enum_only().symbol().string();
            let obj = self.value.as_object().unwrap();
            let properties: ObjectIter<'_, _, Value<'_>> =
                obj.own_props::<Value<'_>, Value<'_>>(filter);
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
        if self.value.is_null() || self.value.is_undefined() {
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
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    properties: ObjectIter<'de, Value<'de>, Value<'de>>,
}

impl<'a, 'de> de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(kv) = self.properties.next() {
            let (k, v) = kv.map_err(|e| from_js_error(self.de.value.ctx().clone(), e))?;
            self.de.value = k.clone();
            self.de.map_key = true;
            self.de.current_kv = Some((k, v));
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.value = self.de.current_kv.clone().unwrap().1;
        seed.deserialize(&mut *self.de)
    }
}

struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    seq: Array<'de>,
    length: usize,
    index: usize,
}

impl<'a, 'de> de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index < self.length {
            self.de.value = self
                .seq
                .get(self.index)
                .map_err(|e| from_js_error(self.seq.ctx().clone(), e))?;
            self.index += 1;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Deserializer as ValueDeserializer;
    use crate::{quickjs::Value, serde::MAX_SAFE_INTEGER, Runtime};
    use serde::de::DeserializeOwned;

    fn deserialize_value<T>(v: Value<'_>) -> T
    where
        T: DeserializeOwned,
    {
        let mut deserializer = ValueDeserializer::from(v);
        T::deserialize(&mut deserializer).unwrap()
    }

    #[test]
    fn test_null() {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            let val = Value::new_null(cx);
            deserialize_value::<()>(val);
        });
    }

    #[test]
    fn test_undefined() {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            let val = Value::new_undefined(cx);
            deserialize_value::<()>(val);
        });
    }

    #[test]
    fn test_nan() {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            let val = Value::new_float(cx, f64::NAN);
            let actual = deserialize_value::<f64>(val);
            assert!(actual.is_nan());
        });
    }

    #[test]
    fn test_infinity() {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let val = Value::new_float(cx, f64::INFINITY);
            let actual = deserialize_value::<f64>(val);
            assert!(actual.is_infinite() && actual.is_sign_positive());
        });
    }

    #[test]
    fn test_negative_infinity() {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            let val = Value::new_float(cx, f64::NEG_INFINITY);
            let actual = deserialize_value::<f64>(val);
            assert!(actual.is_infinite() && actual.is_sign_negative());
        })
    }

    #[test]
    fn test_map_always_converts_keys_to_string() {
        let rt = Runtime::default();
        // Sanity check to make sure the quickjs VM always store object
        // object keys as a string an not a numerical value.
        rt.context().with(|c| {
            c.eval::<Value<'_>, _>("var a = {1337: 42};").unwrap();
            let val = c.globals().get("a").unwrap();
            let actual = deserialize_value::<BTreeMap<String, i32>>(val);

            assert_eq!(42, *actual.get("1337").unwrap())
        });
    }

    #[test]
    #[should_panic]
    fn test_map_does_not_support_non_string_keys() {
        let rt = Runtime::default();
        // Sanity check to make sure it's not possible to deserialize
        // to a map where keys are not strings (e.g. numerical value).
        rt.context().with(|c| {
            c.eval::<Value<'_>, _>("var a = {1337: 42};").unwrap();
            let val = c.globals().get("a").unwrap();
            deserialize_value::<BTreeMap<String, i32>>(val);
        });
    }

    #[test]
    fn test_u64_bounds() {
        let rt = Runtime::default();
        rt.context().with(|c| {
            let max = u64::MAX;
            let val = Value::new_number(c.clone(), max as f64);
            let actual = deserialize_value::<f64>(val);
            assert_eq!(max as f64, actual);

            let min = u64::MIN;
            let val = Value::new_number(c.clone(), min as f64);
            let actual = deserialize_value::<f64>(val);
            assert_eq!(min as f64, actual);
        });
    }

    #[test]
    fn test_i64_bounds() {
        let rt = Runtime::default();

        rt.context().with(|c| {
            let max = i64::MAX;
            let val = Value::new_number(c.clone(), max as _);
            let actual = deserialize_value::<f64>(val);
            assert_eq!(max as f64, actual);

            let min = i64::MIN;
            let val = Value::new_number(c.clone(), min as _);
            let actual = deserialize_value::<f64>(val);
            assert_eq!(min as f64, actual);
        });
    }

    #[test]
    fn test_float_to_integer_conversion() {
        let rt = Runtime::default();

        rt.context().with(|c| {
            let expected = MAX_SAFE_INTEGER - 1;
            let val = Value::new_float(c.clone(), expected as _);
            let actual = deserialize_value::<i64>(val);
            assert_eq!(expected, actual);

            let expected = MAX_SAFE_INTEGER + 1;
            let val = Value::new_float(c.clone(), expected as _);
            let actual = deserialize_value::<f64>(val);
            assert_eq!(expected as f64, actual);
        });
    }

    #[test]
    fn test_u32_upper_bound() {
        let rt = Runtime::default();

        rt.context().with(|c| {
            let expected = u32::MAX;
            let val = Value::new_number(c, expected as _);
            let actual = deserialize_value::<u32>(val);
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_u32_lower_bound() {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let expected = i32::MAX as u32 + 1;
            let val = Value::new_number(cx, expected as _);
            let actual = deserialize_value::<u32>(val);
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_array() {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            cx.eval::<Value<'_>, _>("var a = [1, 2, 3];").unwrap();
            let v = cx.globals().get("a").unwrap();

            let val = deserialize_value::<Vec<u8>>(v);

            assert_eq!(vec![1, 2, 3], val);
        });
    }
}
