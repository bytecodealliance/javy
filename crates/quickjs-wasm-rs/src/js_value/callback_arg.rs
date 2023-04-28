use std::{collections::HashMap, convert::TryInto, fmt};

use anyhow::Result;

use super::{qjs_convert::from_qjs_value, JSValue};
use crate::js_binding::value::JSValueRef;

/// `CallbackArg` is given  to callback functions and can then be lazily evaluated to a `JSValue`.
///
/// # Example
/// ```
/// let context = JSContextRef::default();
/// context.wrap_callback(|_ctx, _this, args| {
///     // Convert args[0] to a Rust `String` type
///     let s: String = args[0].try_into()?;
///     println!("{}", s);
///     Ok(JSValue::Undefined)
/// })?;
/// ```
#[derive(Copy, Clone)]
pub struct CallbackArg<'a> {
    inner: JSValueRef<'a>,
}

impl<'a> CallbackArg<'a> {
    /// Create a new `CallbackArg` with a `JSValueRef`.
    pub fn new(inner: JSValueRef<'a>) -> CallbackArg<'a> {
        Self { inner }
    }

    /// Get the underlying `JSValueRef` value wrapped by this `CallbackArg`. This is used as an escape hatch to operate directly with the raw pointer to the QuickJS value.
    ///
    /// # Safety
    ///
    /// This function is marked unsafe because `JSValueRef` contains a raw pointer to the underlying QuickJS value so it is possible to cause undefined behavior if the pointer is used incorrectly.
    pub unsafe fn inner_value(&self) -> JSValueRef {
        self.inner
    }

    /// Convert the underlying `JSValueRef` to a Rust `JSValue` type.
    fn to_js_value(self) -> Result<JSValue> {
        from_qjs_value(&self.inner)
    }
}

impl fmt::Display for CallbackArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_js_value().unwrap())
    }
}

/// A macro to implement the `TryFrom<&CallbackArg>` and `TryFrom<CallbackArg>` trait
/// for various Rust types.
macro_rules! try_from_impl {
    ($($t:ty),+ $(,)?) => {
        $(impl TryFrom<&CallbackArg<'_>> for $t {
            type Error = anyhow::Error;

            fn try_from(value: &CallbackArg) -> Result<Self> {
                value.to_js_value()?.try_into()
            }
        }

        impl TryFrom<CallbackArg<'_>> for $t {
            type Error = anyhow::Error;

            fn try_from(value: CallbackArg) -> Result<Self> {
                value.to_js_value()?.try_into()
            }
        })+
    };
}

try_from_impl!(
    bool,
    i32,
    usize,
    f64,
    String,
    Vec<JSValue>,
    Vec<u8>,
    HashMap<String, JSValue>,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js_binding::context::JSContextRef;

    #[test]
    fn test_bool() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "true")?;

        let callback_arg = CallbackArg::new(val);
        assert_eq!("true", callback_arg.to_string());
        let arg: bool = callback_arg.try_into()?;
        assert!(arg);

        let callback_arg_ref = &callback_arg;
        let arg: bool = callback_arg_ref.try_into()?;
        assert!(arg);
        Ok(())
    }

    #[test]
    fn test_i32() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42")?;

        let callback_arg = CallbackArg::new(val);
        assert_eq!("42", callback_arg.to_string());
        let arg: i32 = callback_arg.try_into()?;
        assert_eq!(42, arg);

        let callback_arg_ref = &callback_arg;
        let arg: i32 = callback_arg_ref.try_into()?;
        assert_eq!(42, arg);
        Ok(())
    }

    #[test]
    fn test_usize() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42")?;

        let callback_arg = CallbackArg::new(val);
        assert_eq!("42", callback_arg.to_string());
        let arg: usize = callback_arg.try_into()?;
        assert_eq!(42, arg);

        let callback_arg_ref = &callback_arg;
        let arg: usize = callback_arg_ref.try_into()?;
        assert_eq!(42, arg);
        Ok(())
    }

    #[test]
    fn test_f64() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42.42")?;

        let callback_arg = CallbackArg::new(val);
        assert_eq!("42.42", callback_arg.to_string());
        let arg: f64 = callback_arg.try_into()?;
        assert_eq!(42.42, arg);

        let callback_arg_ref = &callback_arg;
        let arg: f64 = callback_arg_ref.try_into()?;
        assert_eq!(42.42, arg);
        Ok(())
    }

    #[test]
    fn test_string() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "const h = 'hello'; h")?;

        let callback_arg = CallbackArg::new(val);
        assert_eq!("hello", callback_arg.to_string());
        let arg: String = callback_arg.try_into()?;
        assert_eq!("hello", arg);

        let callback_arg_ref = &callback_arg;
        let arg: String = callback_arg_ref.try_into()?;
        assert_eq!("hello", arg);
        Ok(())
    }

    #[test]
    fn test_vec() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "[1, 2, 3]")?;

        let expected: Vec<JSValue> = vec![1.into(), 2.into(), 3.into()];

        let callback_arg = CallbackArg::new(val);
        assert_eq!("1,2,3", callback_arg.to_string());
        let arg: Vec<JSValue> = callback_arg.try_into()?;
        assert_eq!(expected, arg);

        let callback_arg_ref = &callback_arg;
        let arg: Vec<JSValue> = callback_arg_ref.try_into()?;
        assert_eq!(expected, arg);
        Ok(())
    }

    #[test]
    fn test_bytes() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "new ArrayBuffer(8)")?;

        let expected = [0_u8; 8].to_vec();

        let callback_arg = CallbackArg::new(val);
        assert_eq!("[object ArrayBuffer]", callback_arg.to_string());
        let arg: Vec<u8> = callback_arg.try_into()?;
        assert_eq!(expected, arg);

        let callback_arg_ref = &callback_arg;
        let arg: Vec<u8> = callback_arg_ref.try_into()?;
        assert_eq!(expected, arg);
        Ok(())
    }

    #[test]
    fn test_hashmap() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "({a: 1, b: 2, c: 3})")?;

        let expected = HashMap::from([
            ("a".to_string(), 1.into()),
            ("b".to_string(), 2.into()),
            ("c".to_string(), 3.into()),
        ]);

        let callback_arg = CallbackArg::new(val);
        assert_eq!("[object Object]", callback_arg.to_string());
        let arg: HashMap<String, JSValue> = callback_arg.try_into()?;
        assert_eq!(expected, arg);

        let callback_arg_ref = &callback_arg;
        let arg: HashMap<String, JSValue> = callback_arg_ref.try_into()?;
        assert_eq!(expected, arg);
        Ok(())
    }
}
