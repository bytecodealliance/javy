use super::{context::JSContextRef, exception::Exception, value::JSValueRef};
use anyhow::Result;
use quickjs_wasm_sys::{
    JSAtom, JSPropertyEnum, JSValue, JS_AtomToString, JS_GetOwnPropertyNames,
    JS_GetPropertyInternal, JS_GPN_ENUM_ONLY, JS_GPN_STRING_MASK, JS_GPN_SYMBOL_MASK,
};
use std::ptr;

#[derive(Debug)]
pub struct Properties<'a> {
    value: JSValue,
    context: &'a JSContextRef,
    property_enum: *mut JSPropertyEnum,
    current_key: JSAtom,
    length: isize,
    offset: isize,
}

impl<'a> Properties<'a> {
    pub(super) fn new(context: &'a JSContextRef, value: JSValue) -> Result<Self> {
        let flags = (JS_GPN_STRING_MASK | JS_GPN_SYMBOL_MASK | JS_GPN_ENUM_ONLY) as i32;
        let mut property_enum: *mut JSPropertyEnum = ptr::null_mut();
        let mut length = 0;
        let ret = unsafe {
            JS_GetOwnPropertyNames(context.inner, &mut property_enum, &mut length, value, flags)
        };

        if ret < 0 {
            let exception = Exception::new(context)?;
            return Err(exception.into_error());
        }

        Ok(Self {
            value,
            context,
            property_enum,
            length: length as isize,
            offset: 0,
            current_key: 0_u32,
        })
    }

    pub fn next_key(&mut self) -> Result<Option<JSValueRef<'a>>> {
        if self.offset >= self.length {
            Ok(None)
        } else {
            let prop_enum = unsafe { self.property_enum.offset(self.offset) };
            self.offset += 1;
            self.current_key = unsafe { (*prop_enum).atom };
            Ok(self.atom_to_string(self.current_key).ok())
        }
    }

    pub fn next_value(&self) -> Result<JSValueRef<'a>> {
        let val = unsafe {
            JS_GetPropertyInternal(
                self.context.inner,
                self.value,
                self.current_key,
                self.value,
                0,
            )
        };
        JSValueRef::new(self.context, val)
    }

    fn atom_to_string(&self, atom: JSAtom) -> Result<JSValueRef<'a>> {
        let raw = unsafe { JS_AtomToString(self.context.inner, atom) };
        JSValueRef::new(self.context, raw)
    }
}

#[cfg(test)]
mod tests {
    use super::super::context::JSContextRef;
    use anyhow::Result;

    #[test]
    fn test_keys() -> Result<()> {
        let contents = "globalThis.o = {a: 1, b: 2, c: [1, 2, 3]};";
        let context = JSContextRef::default();
        context.eval_global("script", contents)?;
        let global = context.global_object()?;
        let o = global.get_property("o")?;

        let mut props = o.properties()?;
        let a = props.next_key()?.unwrap();
        assert!(a.is_str());

        let b = props.next_key()?.unwrap();
        assert!(b.is_str());

        let c = props.next_key()?.unwrap();
        assert!(c.is_str());

        let d = props.next_key()?;
        assert!(d.is_none());

        Ok(())
    }

    #[test]
    fn test_values() -> Result<()> {
        let contents = "globalThis.o = {a: 1, b: 2, c: [1, 2, 3]};";
        let context = JSContextRef::default();
        context.eval_global("script", contents)?;
        let global = context.global_object()?;
        let o = global.get_property("o")?;

        let mut props = o.properties()?;
        props.next_key()?;
        let a = props.next_value()?;
        assert!(a.is_repr_as_i32());

        props.next_key()?;
        let b = props.next_value()?;
        assert!(b.is_repr_as_i32());

        props.next_key()?;
        let c = props.next_value()?;
        assert!(c.is_array());

        Ok(())
    }

    #[test]
    fn test_invalid_access_to_own_props() {
        let context = JSContextRef::default();
        let val = context.value_from_i32(1_i32).unwrap();
        let err = val.properties().unwrap_err();
        assert_eq!(
            "Uncaught TypeError: not an object\n".to_string(),
            err.to_string()
        );
    }
}
