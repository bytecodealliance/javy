use super::value::Value;
use anyhow::{anyhow, Result};
use quickjs_sys::{
    JSAtom, JSContext, JSPropertyEnum, JSValue, JS_AtomToString, JS_GetOwnPropertyNames,
    JS_GetPropertyInternal, JS_GPN_ENUM_ONLY, JS_GPN_STRING_MASK, JS_GPN_SYMBOL_MASK,
};
use std::ptr;

pub struct OwnProperties {
    value: JSValue,
    context: *mut JSContext,
    property_enum: *mut JSPropertyEnum,
    current_key: JSAtom,
    length: isize,
    offset: isize,
}

impl OwnProperties {
    pub fn from(value: &Value) -> Result<Self> {
        let flags = (JS_GPN_STRING_MASK | JS_GPN_SYMBOL_MASK | JS_GPN_ENUM_ONLY) as i32;
        let mut property_enum: *mut JSPropertyEnum = ptr::null_mut();
        let mut length = 0;
        let result = unsafe {
            JS_GetOwnPropertyNames(
                value.inner_context(),
                &mut property_enum,
                &mut length,
                value.inner(),
                flags,
            )
        };

        // TODO: Exception handling for non-nan-boxed values
        if result != 1 {
            Err(anyhow!(
                "Couldn't retrieve own properties from: {:?}",
                value
            ))
        } else {
            Ok(Self {
                value: value.inner(),
                context: value.inner_context(),
                property_enum,
                length: length as isize,
                offset: 0,
                current_key: 0_u32,
            })
        }
    }

    pub fn next_key(&mut self) -> Result<Option<Value>> {
        if self.offset >= self.length {
            Ok(None)
        } else {
            let prop_enum = unsafe { self.property_enum.offset(self.offset) };
            self.offset += 1;
            self.current_key = unsafe { (*prop_enum).atom };
            Ok(self.atom_to_string(self.current_key).ok())
        }
    }

    pub fn next_value(&self) -> Result<Value> {
        let val = unsafe {
            JS_GetPropertyInternal(self.context, self.value, self.current_key, self.value, 0)
        };
        Value::new(self.context, val)
    }

    fn atom_to_string(&self, atom: JSAtom) -> Result<Value> {
        let raw = unsafe { JS_AtomToString(self.context, atom) };
        Value::new(self.context, raw)
    }
}
