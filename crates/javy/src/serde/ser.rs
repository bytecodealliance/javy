use crate::quickjs::{object::Property, Array, Ctx, Object, String as JSString, Value};
use crate::serde::err::{Error, Result};
use anyhow::anyhow;

use serde::{ser, ser::Error as SerError, Serialize};

/// `Serializer` is a serializer for [Value] values, implementing the `serde::Serializer` trait.
///
/// This struct is responsible for converting Rust types into [Value] using the Serde
/// serialization framework.
///
/// ```
/// // Assuming you have [`Ctx`] instance named context
/// let serializer = Serializer::from_context(context)?;
/// let value: Value = serializer.serialize_u32(42)?;
/// ```
pub struct Serializer<'js> {
    pub context: Ctx<'js>,
    pub value: Value<'js>,
    pub key: Value<'js>,
}

impl SerError for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(anyhow!(msg.to_string()))
    }
}

impl<'js> Serializer<'js> {
    pub fn from_context(context: Ctx<'js>) -> Result<Self> {
        Ok(Self {
            context: context.clone(),
            value: Value::new_undefined(context.clone()),
            key: Value::new_undefined(context),
        })
    }
}

impl ser::Serializer for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.value = Value::new_int(self.context.clone(), v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.value = Value::new_number(self.context.clone(), v as _);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        // NOTE: See optimization note in serialize_f64.
        self.serialize_f64(f64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.value = Value::new_number(self.context.clone(), v as _);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        // NOTE: See optimization note in serialize_f64.
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        // NOTE: QuickJS will create a number value backed by an i32 when the value is within
        // the i32::MIN..=i32::MAX as an optimization. Otherwise the value will be backed by a f64.
        self.value = Value::new_float(self.context.clone(), v);
        Ok(())
    }

    fn serialize_bool(self, b: bool) -> Result<()> {
        self.value = Value::new_bool(self.context.clone(), b);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let js_string = JSString::from_str(self.context.clone(), v).map_err(Error::custom)?;
        self.value = Value::from(js_string);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit(self) -> Result<()> {
        self.value = Value::new_null(self.context.clone());
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        let arr = Array::new(self.context.clone()).map_err(Error::custom)?;
        self.value = arr.into_value();
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        let obj = Object::new(self.context.clone()).map_err(Error::custom)?;
        self.value = Value::from(obj);
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_map(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_map(Some(len))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let obj = Object::new(self.context.clone()).map_err(Error::custom)?;
        value.serialize(&mut *self)?;
        obj.set(variant, self.value.clone())
            .map_err(Error::custom)?;
        self.value = Value::from(obj);

        Ok(())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        Err(Error::custom("Cannot serialize bytes"))
    }
}

impl ser::SerializeSeq for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut element_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut element_serializer)?;

        if let Some(v) = self.value.as_array() {
            return v
                .set(v.len(), element_serializer.value.clone())
                .map_err(|e| Error::custom(e.to_string()));
        }
        Err(Error::custom("Expected to be an array"))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut element_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut element_serializer)?;

        if let Some(v) = self.value.as_array() {
            return v
                .set(v.len(), element_serializer.value.clone())
                .map_err(|e| Error::custom(e.to_string()));
        }

        Err(Error::custom("Expected to be an array"))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut field_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut field_serializer)?;
        if let Some(v) = self.value.as_array() {
            return v
                .set(v.len(), field_serializer.value.clone())
                .map_err(|e| Error::custom(e.to_string()));
        }

        Err(Error::custom("Expected to be an array"))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut field_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut field_serializer)?;

        if let Some(v) = self.value.as_array() {
            return v
                .set(v.len(), field_serializer.value.clone())
                .map_err(|e| Error::custom(e.to_string()));
        }

        Err(Error::custom("Expected to be an array"))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut key_serializer = Serializer::from_context(self.context.clone())?;
        key.serialize(&mut key_serializer)?;
        self.key = key_serializer.value;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut map_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut map_serializer)?;
        if let Some(o) = self.value.as_object() {
            let prop = Property::from(map_serializer.value.clone())
                .writable()
                .configurable()
                .enumerable();
            o.prop::<_, _, _>(self.key.clone(), prop)
                .map_err(|e| Error::custom(e.to_string()))
        } else {
            Err(Error::custom("Expected to be an object"))
        }
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut field_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut field_serializer)?;

        if let Some(o) = self.value.as_object() {
            return o
                .set(key, field_serializer.value.clone())
                .map_err(|e| Error::custom(e.to_string()));
        }

        Err(Error::custom("Expected to be an object"))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut field_serializer = Serializer::from_context(self.context.clone())?;
        value.serialize(&mut field_serializer)?;

        if let Some(o) = self.value.as_object() {
            return o
                .set(key, field_serializer.value.clone())
                .map_err(|e| Error::custom(e.to_string()));
        }

        Err(Error::custom("Expected to be an object"))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Serializer as ValueSerializer;
    use crate::serde::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER};
    use crate::Runtime;
    use anyhow::Result;
    use quickcheck::quickcheck;
    use serde::{Serialize, Serializer};

    fn with_serializer<F: FnMut(&mut ValueSerializer) -> Result<bool>>(
        rt: &Runtime,
        mut w: F,
    ) -> Result<bool> {
        rt.context().with(|c| {
            let mut serializer = ValueSerializer::from_context(c.clone()).unwrap();
            w(&mut serializer)
        })
    }

    quickcheck! {
        fn test_i16(v: i16) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_i16(v)?;
                Ok(serializer.value.is_int())
            })
        }

        fn test_i32(v: i32) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_i32(v)?;
                Ok(serializer.value.is_int())
            })
        }

        fn test_i64(v: i64) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                if (MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&v) {
                    serializer.serialize_i64(v)?;
                    Ok(serializer.value.is_number())
                } else {
                    serializer.serialize_f64(v as f64)?;
                    Ok(serializer.value.is_number())
                }
            })
        }

        fn test_u64(v: u64) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                if v <= MAX_SAFE_INTEGER as u64 {
                    serializer.serialize_u64(v)?;
                    Ok(serializer.value.is_number())
                } else {
                    serializer.serialize_f64(v as f64)?;
                    Ok(serializer.value.is_number())
                }
            })
        }

        fn test_u16(v: u16) -> Result<bool> {
            let rt = Runtime::default();

            with_serializer(&rt, |serializer| {
                serializer.serialize_u16(v)?;
                Ok(serializer.value.is_int())
            })
        }

        fn test_u32(v: u32) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_u32(v)?;
                // QuickJS optimizes numbers in the range of [i32::MIN..=i32::MAX]
                // as ints
                if v > i32::MAX as u32 {
                    Ok(serializer.value.is_float())
                } else {
                    Ok(serializer.value.is_int())
                }
            })

        }

        fn test_f32(v: f32) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_f32(v)?;

                if v == 0.0_f32 {
                    if v.is_sign_positive() {
                        return  Ok(serializer.value.is_int());
                    }


                    if v.is_sign_negative() {
                        return Ok(serializer.value.is_float());
                    }
                }

                // The same (int) optimization is happening at this point,
                // but here we need to account for signs
                let zero_fractional_part = v.fract() == 0.0;
                let range = (i32::MIN as f32)..=(i32::MAX as f32);

                if zero_fractional_part && range.contains(&v) {
                    Ok(serializer.value.is_int())
                } else {
                    Ok(serializer.value.is_float())
                }
            })
        }

        fn test_f64(v: f64) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_f64(v)?;

                if v == 0.0_f64 {
                    if v.is_sign_positive() {
                        return  Ok(serializer.value.is_int());
                    }


                    if v.is_sign_negative() {
                        return Ok(serializer.value.is_float());
                    }
                }

                // The same (int) optimization is happening at this point,
                // but here we need to account for signs
                let zero_fractional_part = v.fract() == 0.0;
                let range = (i32::MIN as f64)..=(i32::MAX as f64);

                if zero_fractional_part && range.contains(&v) {
                    Ok(serializer.value.is_int())
                } else {
                    Ok(serializer.value.is_float())
                }
            })
        }

        fn test_bool(v: bool) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_bool(v)?;
                Ok(serializer.value.is_bool())
            })
        }

        fn test_str(v: String) -> Result<bool> {
            let rt = Runtime::default();
            with_serializer(&rt, |serializer| {
                serializer.serialize_str(v.as_str())?;

                Ok(serializer.value.is_string())
            })
        }
    }

    #[test]
    fn test_null() -> Result<()> {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();
            serializer.serialize_unit().unwrap();

            assert!(serializer.value.is_null());
        });
        Ok(())
    }

    #[test]
    fn test_nan() -> Result<()> {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();
            serializer.serialize_f64(f64::NAN).unwrap();
            assert!(serializer.value.is_number());
        });
        Ok(())
    }

    #[test]
    fn test_infinity() -> Result<()> {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();
            serializer.serialize_f64(f64::INFINITY).unwrap();
            assert!(serializer.value.is_number());
        });
        Ok(())
    }

    #[test]
    fn test_negative_infinity() -> Result<()> {
        let rt = Runtime::default();
        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();
            serializer.serialize_f64(f64::NEG_INFINITY).unwrap();
            assert!(serializer.value.is_number());
        });
        Ok(())
    }

    #[test]
    fn test_map() {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();

            let mut map = BTreeMap::new();
            map.insert("foo", "bar");
            map.insert("toto", "titi");

            map.serialize(&mut serializer).unwrap();

            assert!(serializer.value.is_object())
        });
    }

    #[test]
    fn test_struct_into_map() {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();

            #[derive(serde::Serialize)]
            struct MyObject {
                foo: String,
                bar: u32,
            }

            let my_object = MyObject {
                foo: "hello".to_string(),
                bar: 1337,
            };
            my_object.serialize(&mut serializer).unwrap();

            assert!(serializer.value.is_object());
        });
    }

    #[test]
    fn test_sequence() {
        let rt = Runtime::default();

        rt.context().with(|cx| {
            let mut serializer = ValueSerializer::from_context(cx.clone()).unwrap();

            let sequence = vec!["hello", "world"];

            sequence.serialize(&mut serializer).unwrap();

            assert!(serializer.value.is_array());
        });
    }
}
