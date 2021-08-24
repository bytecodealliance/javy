use crate::context::Context;
use quickjs_sys as q;
use serde::{de, ser, Serialize};
use std::{
    fmt::{self, Display},
    ptr,
};

pub struct Serializer {
    pub context: Context,
    pub key: q::JSValue,
    pub value: q::JSValue,
}

pub struct Deserializer<'de> {
    pub context: &'de Context,
    pub value: q::JSValue,
    pub atom: q::JSAtom,
    pub props: *mut q::JSPropertyEnum,
    pub len: isize,
    pub offset: isize,
}

impl Serializer {
    pub fn from_context(context: Context) -> Self {
        Self {
            key: 0_u64,
            value: 0_u64,
            context,
        }
    }
}

impl<'de> Deserializer<'de> {
    pub fn from(context: &'de Context, value: q::JSValue) -> Self {
        Self {
            context,
            value,
            props: ptr::null_mut(),
            len: 0,
            offset: 0,
            atom: 0,
        }
    }

    pub fn with_props(&mut self, props: *mut q::JSPropertyEnum, len: isize) -> &mut Self {
        self.props = props;
        self.len = len;
        self
    }

    pub fn next_key(&mut self) -> Option<q::JSValue> {
        if self.offset >= self.len {
            return None;
        }

        unsafe {
            let js_prop_enum = self.props.offset(self.offset);
            self.offset += 1;
            self.atom = (*js_prop_enum).atom;
            Some(self.context.atom_to_string(self.atom))
        }
    }

    pub fn next_value(&mut self) -> Result<q::JSValue> {
        let val = self.context.get_internal_property(self.value, self.atom);
        Ok(val)
    }

    pub fn next_element(&mut self) -> Option<q::JSValue> {
        if self.offset >= self.len {
            return None;
        }
        let val = self
            .context
            .get_uint32_property(self.value, self.offset as u32);
        self.offset += 1;
        Some(val)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Ints and Floats

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.value = unsafe { self.context.new_int32(v) };
        Ok(())
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        // big int
        unreachable!()
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.value = unsafe { self.context.new_float64(v as f64) };
        Ok(())
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        // big int
        unreachable!()
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.value = unsafe { self.context.new_float64(v) };
        Ok(())
    }

    // Boolean

    fn serialize_bool(self, b: bool) -> Result<()> {
        self.value = unsafe { self.context.new_bool(b) };

        Ok(())
    }

    // Strings

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.value = self.context.new_string(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // Null

    fn serialize_unit(self) -> Result<()> {
        self.value = (q::JS_TAG_NULL as u64) << 32;
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

    // Arrays

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.value = self.context.new_array();
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

    // Maps

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.value = self.context.new_object();
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
        let obj = self.context.new_object();
        self.value = {
            value.serialize(&mut *self)?;
            self.context.set_str_property(obj, variant, self.value);
            obj
        };

        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        // Should never happen
        unimplemented!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let seq = self.value;
        value.serialize(&mut **self)?;
        self.context.set_uint32_property(seq, self.value);
        self.value = seq;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let seq = self.value;
        value.serialize(&mut **self)?;
        self.context.set_uint32_property(seq, self.value);
        self.value = seq;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let seq = self.value;
        value.serialize(&mut **self)?;
        self.context.set_uint32_property(seq, self.value);
        self.value = seq;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let seq = self.value;
        value.serialize(&mut **self)?;
        self.context.set_uint32_property(seq, self.value);
        self.value = seq;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let obj = self.value;
        key.serialize(&mut **self)?;
        self.key = self.value;
        self.value = obj;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut map_serializer = Serializer::from_context(self.context);
        value.serialize(&mut map_serializer)?;
        let key_name = self.context.to_byte_slice(self.key);
        self.context
            .set_property_raw(self.value, key_name.as_ptr() as *const _, map_serializer.value);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let obj = self.value;
        value.serialize(&mut **self)?;
        let v = self.value;
        self.context.set_str_property(obj, key, v);
        self.value = obj;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let obj = self.value;
        value.serialize(&mut **self)?;
        let v = self.value;
        self.context.set_str_property(obj, key, v);
        self.value = obj;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.context.get_tag(self.value) as i32 {
            q::JS_TAG_INT => self.deserialize_i32(visitor),
            q::JS_TAG_BOOL => self.deserialize_bool(visitor),
            q::JS_TAG_NULL | q::JS_TAG_UNDEFINED => self.deserialize_unit(visitor),
            q::JS_TAG_STRING => self.deserialize_str(visitor),
            q::JS_TAG_OBJECT => {
                if self.context.is_array(self.value) {
                    self.deserialize_seq(visitor)
                } else {
                    self.deserialize_map(visitor)
                }
            }
            tag => {
                // u32 are also serialized as f64;
                // here it's worth checking if `self.value` is actually a f32 or f64
                // if it is not, then it can safely be deserialized as u32
                if unsafe { self.context.is_float64(self.value) } {
                    return self.deserialize_f64(visitor);
                }

                println!("TAG: {:?}", tag);

                Err(Error::Message("Couldn't deserialize value".to_string()))
            }
        }
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
        visitor.visit_i32(self.value as i32)
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
        let val = unsafe { self.context.to_float64(self.value) };
        visitor.visit_f64(val)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.value as i32 > 0)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let string = self.context.deserialize_string(self.value);
        visitor.visit_str(&string)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
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
        let len = self.context.get_str_property("length", self.value);
        visitor.visit_seq(self.with_props(ptr::null_mut(), len as isize))
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
        let (props, len) = self.context.get_own_properties(self.value);
        visitor.visit_map(self.with_props(props, len as isize))
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

impl<'de> de::MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(k) = self.next_key() {
            let mut key_deserializer = Deserializer::from(&self.context, k);
            return seed.deserialize(&mut key_deserializer).map(Some);
        }

        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Ok(p) = self.next_value() {
            let mut prop_deserializer = Deserializer::from(&self.context, p);
            return seed.deserialize(&mut prop_deserializer);
        }

        Err(Error::Message("Error deserializing value".to_string()))
    }
}

impl<'de> de::SeqAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(e) = self.next_element() {
            let mut element_deserializer = Deserializer::from(&self.context, e);
            return seed.deserialize(&mut element_deserializer).map(Some);
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    mod ser {
        use crate::serialization::Serializer as ValueSerializer;
        use crate::Context;
        use anyhow::Result;
        use quickcheck::quickcheck;
        use quickjs_sys as q;
        use serde::Serializer;

        quickcheck! {
            fn test_i16(v: i16) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);
                serializer.serialize_i16(v)?;
                Ok(context.get_tag(serializer.value) == q::JS_TAG_INT as u64)
            }

            fn test_i32(v: i32) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);
                serializer.serialize_i32(v)?;
                Ok(context.get_tag(serializer.value) == q::JS_TAG_INT as u64)
            }

            fn test_u16(v: u16) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);

                serializer.serialize_u16(v)?;

                Ok(context.get_tag(serializer.value) == q::JS_TAG_INT as u64)
            }

            fn test_u32(v: u32) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);

                serializer.serialize_u32(v)?;
                // QuickJS optimizes numbers in the range of [i32::MIN..=i32::MAX]
                // as ints
                if v > i32::MAX as u32 {
                    Ok(unsafe {context.is_float64(serializer.value) })
                } else {
                    Ok(context.get_tag(serializer.value) == q::JS_TAG_INT as u64)
                }
            }

            fn test_f64(v: f64) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);
                serializer.serialize_f64(v)?;

                if v == 0.0_f64 {
                    if v.is_sign_positive() {
                        return  Ok(context.get_tag(serializer.value) == q::JS_TAG_INT as u64);
                    }


                    if v.is_sign_negative() {
                        return Ok(unsafe { context.is_float64(serializer.value) });
                    }
                }

                // The same (int) optimization is happening at this point,
                // but here we need to account for signs
                let zero_fractional_part = v.fract() == 0.0;
                let range = (i32::MIN as f64)..=(i32::MAX as f64);

                if zero_fractional_part && range.contains(&v) {
                    Ok(context.get_tag(serializer.value) == q::JS_TAG_INT as u64)
                } else {
                    Ok(unsafe { context.is_float64(serializer.value) })
                }
            }

            fn test_bool(v: bool) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);
                serializer.serialize_bool(v)?;

                Ok(context.get_tag(serializer.value) == q::JS_TAG_BOOL as u64)
            }

            fn test_str(v: String) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut serializer = ValueSerializer::from_context(context);
                serializer.serialize_str(v.as_str())?;

                Ok(context.is_string(serializer.value))
            }
        }

        #[test]
        fn test_null() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut serializer = ValueSerializer::from_context(context);
            serializer.serialize_unit()?;

            assert_eq!(context.get_tag(serializer.value), q::JS_TAG_NULL as u64);
            Ok(())
        }

        #[test]
        fn test_nan() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut serializer = ValueSerializer::from_context(context);
            serializer.serialize_f64(f64::NAN)?;

            assert!(unsafe { context.is_float64(serializer.value) });
            Ok(())
        }

        #[test]
        fn test_infinity() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut serializer = ValueSerializer::from_context(context);
            serializer.serialize_f64(f64::INFINITY)?;

            assert!(unsafe { context.is_float64(serializer.value) });
            Ok(())
        }

        #[test]
        fn test_negative_infinity() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut serializer = ValueSerializer::from_context(context);
            serializer.serialize_f64(f64::NEG_INFINITY)?;

            assert!(unsafe { context.is_float64(serializer.value) });
            Ok(())
        }
    }

    mod de {
        use crate::serialization::Deserializer as ValueDeserializer;
        use crate::Context;
        use anyhow::Result;
        use quickcheck::quickcheck;
        use quickjs_sys as q;
        use serde::Deserialize;

        quickcheck! {
            fn test_i32(v: i32) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut deserializer = ValueDeserializer::from(
                    &context,
                    unsafe { context.new_int32(v) }
                );

                let result = i32::deserialize(&mut deserializer)?;
                Ok(result == v)
            }

            fn test_bool(v: bool) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut deserializer = ValueDeserializer::from(
                    &context,
                    unsafe { context.new_bool(v) }
                );

                let result = bool::deserialize(&mut deserializer)?;
                Ok(result == v)
            }

            fn test_str(v: String) -> Result<bool> {
                let context = Context::new().expect("Couldn't create context");
                let mut deserializer = ValueDeserializer::from(
                    &context,
                    context.new_string(v.as_str())
                );

                let result = String::deserialize(&mut deserializer)?;
                Ok(result == v)
            }
        }

        #[test]
        fn test_null() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut deserializer = ValueDeserializer::from(&context, (q::JS_TAG_NULL as u64) << 32);

            type U = ();
            let result = U::deserialize(&mut deserializer)?;
            assert_eq!(result, ());
            Ok(())
        }

        #[test]
        fn test_undefined() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut deserializer =
                ValueDeserializer::from(&context, (q::JS_TAG_UNDEFINED as u64) << 32);

            type U = ();
            let result = U::deserialize(&mut deserializer)?;
            assert_eq!(result, ());
            Ok(())
        }

        #[test]
        fn test_nan() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut deserializer =
                ValueDeserializer::from(&context, unsafe { context.new_float64(f64::NAN) });

            let result = f64::deserialize(&mut deserializer)?;
            assert!(result.is_nan());
            Ok(())
        }

        #[test]
        fn test_infinity() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut deserializer =
                ValueDeserializer::from(&context, unsafe { context.new_float64(f64::INFINITY) });

            let result = f64::deserialize(&mut deserializer)?;
            assert!(result.is_infinite() && result.is_sign_positive());
            Ok(())
        }

        #[test]
        fn test_negative_infinity() -> Result<()> {
            let context = Context::new().expect("Couldn't create context");
            let mut deserializer = ValueDeserializer::from(&context, unsafe {
                context.new_float64(f64::NEG_INFINITY)
            });

            let result = f64::deserialize(&mut deserializer)?;
            assert!(result.is_infinite() && result.is_sign_negative());
            Ok(())
        }
    }

    mod roundtrips {
        use anyhow::Result;
        use crate::serialization::{Serializer as ValueSerializer, Deserializer as ValueDeserializer};
        use crate::Context;
        use serde::{Serializer, Deserialize};
        use quickcheck::quickcheck;

        quickcheck! {
            fn test_str(v: String) -> Result<bool> {
                let context = Context::new().unwrap();
                let mut serializer = ValueSerializer::from_context(context);
                serializer.serialize_str(v.as_str()).unwrap();

                let context = Context::new().unwrap();
                let mut deserializer = ValueDeserializer::from(&context, serializer.value);

                let result = String::deserialize(&mut deserializer).unwrap();
                Ok(v == result)
            }
        }
    }
}
