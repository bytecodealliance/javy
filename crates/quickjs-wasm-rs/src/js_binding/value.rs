use std::{convert::TryInto, fmt};

use super::context::JSContextRef;
use super::exception::Exception;
use super::properties::Properties;
use crate::js_value::{qjs_convert::from_qjs_value, JSValue};
use anyhow::{anyhow, Result};
use quickjs_wasm_sys::{
    size_t as JS_size_t, JSValue as JSValueRaw, JS_BigIntSigned, JS_BigIntToInt64,
    JS_BigIntToUint64, JS_Call, JS_DefinePropertyValueStr, JS_DefinePropertyValueUint32,
    JS_EvalFunction, JS_GetArrayBuffer, JS_GetPropertyStr, JS_GetPropertyUint32, JS_IsArray,
    JS_IsArrayBuffer_Ext, JS_IsFloat64_Ext, JS_IsFunction, JS_ToCStringLen2, JS_ToFloat64,
    JS_PROP_C_W_E, JS_TAG_BIG_INT, JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_INT, JS_TAG_NULL,
    JS_TAG_OBJECT, JS_TAG_STRING, JS_TAG_UNDEFINED,
};
use std::borrow::Cow;
use std::ffi::CString;
use std::str;

#[derive(Debug, PartialEq, Eq)]
pub enum BigInt {
    Signed(i64),
    Unsigned(u64),
}

/// `JSValueRef` is a wrapper around a QuickJS `JSValue` with a reference to its associated `JSContextRef`.
///
/// This struct provides a safe interface for interacting with JavaScript values in the context of
/// their associated QuickJS execution environment.
///
/// # Lifetime
///
/// The lifetime parameter `'a` represents the lifetime of the reference to the `JSContextRef`.
/// This ensures that the `JSValueRef` cannot outlive the context it is associated with, preventing
/// potential use-after-free issues or other unsafe behavior.
#[derive(Debug, Copy, Clone)]
pub struct JSValueRef<'a> {
    pub(super) context: &'a JSContextRef,
    pub(super) value: JSValueRaw,
}

impl<'a> JSValueRef<'a> {
    pub(super) fn new(context: &'a JSContextRef, raw_value: JSValueRaw) -> Result<Self> {
        let value = Self {
            context,
            value: raw_value,
        };

        if value.is_exception() {
            let exception = value.as_exception()?;
            Err(exception.into_error())
        } else {
            Ok(value)
        }
    }

    pub(super) fn new_unchecked(context: &'a JSContextRef, value: JSValueRaw) -> Self {
        Self { context, value }
    }

    pub(super) fn eval_function(&self) -> Result<Self> {
        Self::new(self.context, unsafe {
            JS_EvalFunction(self.context.inner, self.value)
        })
    }

    /// Calls a JavaScript function with the specified `receiver` and `args`.
    ///
    /// # Arguments
    /// * `receiver`: The object on which the function is called.
    /// * `args`: A slice of [`JSValueRef`] representing the arguments to be
    ///   passed to the function.
    pub fn call(&self, receiver: &Self, args: &[Self]) -> Result<Self> {
        let args: Vec<JSValueRaw> = args.iter().map(|v| v.value).collect();
        let return_val = unsafe {
            JS_Call(
                self.context.inner,
                self.value,
                receiver.value,
                args.len() as i32,
                args.as_slice().as_ptr() as *mut JSValueRaw,
            )
        };

        Self::new(self.context, return_val)
    }

    /// Converts the JavaScript value to an `i32` without checking its type.
    pub fn as_i32_unchecked(&self) -> i32 {
        self.value as i32
    }

    /// Converts the JavaScript value to a `u32` without checking its type.
    pub fn as_u32_unchecked(&self) -> u32 {
        self.value as u32
    }

    /// Converts the JavaScript value to an `f64` without checking its type.
    pub fn as_f64_unchecked(&self) -> f64 {
        let mut ret = 0_f64;
        unsafe { JS_ToFloat64(self.context.inner, &mut ret, self.value) };
        ret
    }

    /// Converts the JavaScript value to a `BigInt` without checking its type.
    pub fn as_big_int_unchecked(&self) -> Result<BigInt> {
        if self.is_signed_big_int() {
            let v = self.bigint_as_i64()?;
            Ok(BigInt::Signed(v))
        } else {
            let v = self.bigint_as_u64()?;
            Ok(BigInt::Unsigned(v))
        }
    }

    fn is_signed_big_int(&self) -> bool {
        unsafe { JS_BigIntSigned(self.context.inner, self.value) == 1 }
    }

    fn bigint_as_i64(&self) -> Result<i64> {
        let mut ret = 0_i64;
        let err = unsafe { JS_BigIntToInt64(self.context.inner, &mut ret, self.value) };
        if err < 0 {
            anyhow::bail!("big int underflow, value does not fit in i64");
        }
        Ok(ret)
    }

    fn bigint_as_u64(&self) -> Result<u64> {
        let mut ret = 0_u64;
        let err = unsafe { JS_BigIntToUint64(self.context.inner, &mut ret, self.value) };
        if err < 0 {
            anyhow::bail!("big int overflow, value does not fit in u64");
        }
        Ok(ret)
    }

    /// Checks if the JavaScript value is a number.
    ///
    /// Returns `true` if the value is a number (either represented as an `f64` or an `i32`),
    /// otherwise returns `false`.
    pub fn is_number(&self) -> bool {
        self.is_repr_as_f64() || self.is_repr_as_i32()
    }

    /// Checks if the JavaScript value is a `BigInt`.
    ///
    /// Returns `true` if the value is a `BigInt`, otherwise returns `false`.
    pub fn is_big_int(&self) -> bool {
        self.get_tag() == JS_TAG_BIG_INT
    }

    /// Converts the JavaScript value to an `f64` if it is a number, otherwise returns an error.
    pub fn as_f64(&self) -> Result<f64> {
        if self.is_repr_as_f64() {
            return Ok(self.as_f64_unchecked());
        }
        if self.is_repr_as_i32() {
            return Ok(self.as_i32_unchecked() as f64);
        }
        anyhow::bail!("Value is not a number")
    }

    /// Tries to convert the JavaScript value to an `i32` if it is an integer, otherwise returns an error.
    pub fn try_as_integer(&self) -> Result<i32> {
        if self.is_repr_as_f64() {
            let v = self.as_f64_unchecked();
            if v.trunc() != v {
                anyhow::bail!("Value is not an integer");
            }
            return Ok((v as i64).try_into()?);
        }
        if self.is_repr_as_i32() {
            return Ok(self.as_i32_unchecked());
        }
        anyhow::bail!("Value is not a number")
    }

    /// Converts the JavaScript value to a `bool` if it is a boolean, otherwise returns an error.
    pub fn as_bool(&self) -> Result<bool> {
        if self.is_bool() {
            Ok(self.value as i32 > 0)
        } else {
            Err(anyhow!("Can't represent {:?} as bool", self.value))
        }
    }

    /// Converts the JavaScript value to a string if it is a string.
    pub fn as_str(&self) -> Result<&str> {
        let buffer = self.as_wtf8_str_buffer();
        str::from_utf8(buffer).map_err(Into::into)
    }

    /// Converts the JavaScript value to a string, replacing any invalid UTF-8 sequences with the
    /// Unicode replacement character (U+FFFD).
    pub fn as_str_lossy(&self) -> std::borrow::Cow<str> {
        let mut buffer = self.as_wtf8_str_buffer();
        match str::from_utf8(buffer) {
            Ok(valid) => Cow::Borrowed(valid),
            Err(mut error) => {
                let mut res = String::new();
                loop {
                    let (valid, after_valid) = buffer.split_at(error.valid_up_to());
                    res.push_str(unsafe { str::from_utf8_unchecked(valid) });
                    res.push(char::REPLACEMENT_CHARACTER);

                    // see https://simonsapin.github.io/wtf-8/#surrogate-byte-sequence
                    let lone_surrogate =
                        matches!(after_valid, [0xED, 0xA0..=0xBF, 0x80..=0xBF, ..]);

                    // https://simonsapin.github.io/wtf-8/#converting-wtf-8-utf-8 states that each
                    // 3-byte lone surrogate sequence should be replaced by 1 UTF-8 replacement
                    // char. Rust's `Utf8Error` reports each byte in the lone surrogate byte
                    // sequence as a separate error with an `error_len` of 1. Since we insert a
                    // replacement char for each error, this results in 3 replacement chars being
                    // inserted. So we use an `error_len` of 3 instead of 1 to treat the entire
                    // 3-byte sequence as 1 error instead of as 3 errors and only emit 1
                    // replacement char.
                    let error_len = if lone_surrogate {
                        3
                    } else {
                        error
                            .error_len()
                            .expect("Error length should always be available on underlying buffer")
                    };

                    buffer = &after_valid[error_len..];
                    match str::from_utf8(buffer) {
                        Ok(valid) => {
                            res.push_str(valid);
                            break;
                        }
                        Err(e) => error = e,
                    }
                }
                Cow::Owned(res)
            }
        }
    }

    fn as_wtf8_str_buffer(&self) -> &[u8] {
        unsafe {
            let mut len: JS_size_t = 0;
            let ptr = JS_ToCStringLen2(self.context.inner, &mut len, self.value, 0);
            let ptr = ptr as *const u8;
            let len = len as usize;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    /// Converts the JavaScript value to a byte slice if it is an ArrayBuffer, otherwise returns an error.
    pub fn as_bytes(&self) -> Result<&[u8]> {
        let mut len = 0;
        let ptr = unsafe { JS_GetArrayBuffer(self.context.inner, &mut len, self.value) };
        if ptr.is_null() {
            Err(anyhow!(
                "Can't represent {:?} as an array buffer",
                self.value
            ))
        } else {
            Ok(unsafe { std::slice::from_raw_parts(ptr, len as _) })
        }
    }

    /// Converts the JavaScript value to a mutable byte slice if it is an ArrayBuffer, otherwise returns an error.
    pub fn as_bytes_mut(&self) -> Result<&mut [u8]> {
        let mut len = 0;
        let ptr = unsafe { JS_GetArrayBuffer(self.context.inner, &mut len, self.value) };
        if ptr.is_null() {
            Err(anyhow!(
                "Can't represent {:?} as an array buffer",
                self.value
            ))
        } else {
            Ok(unsafe { std::slice::from_raw_parts_mut(ptr, len as _) })
        }
    }

    /// Retrieves the properties of the JavaScript value.
    pub fn properties(&self) -> Result<Properties<'a>> {
        Properties::new(self.context, self.value)
    }

    /// Checks if the JavaScript value is represented as an `f64`.
    pub fn is_repr_as_f64(&self) -> bool {
        unsafe { JS_IsFloat64_Ext(self.get_tag()) == 1 }
    }

    /// Checks if the JavaScript value is represented as an `i32`.
    pub fn is_repr_as_i32(&self) -> bool {
        self.get_tag() == JS_TAG_INT
    }

    /// Checks if the JavaScript value is a string.
    pub fn is_str(&self) -> bool {
        self.get_tag() == JS_TAG_STRING
    }

    /// Checks if the JavaScript value is a boolean.
    pub fn is_bool(&self) -> bool {
        self.get_tag() == JS_TAG_BOOL
    }

    /// Checks if the JavaScript value is an array.
    pub fn is_array(&self) -> bool {
        unsafe { JS_IsArray(self.context.inner, self.value) == 1 }
    }

    /// Checks if the JavaScript value is an object (excluding arrays).
    pub fn is_object(&self) -> bool {
        !self.is_array() && self.get_tag() == JS_TAG_OBJECT
    }

    /// Checks if the JavaScript value is an ArrayBuffer.
    pub fn is_array_buffer(&self) -> bool {
        (unsafe { JS_IsArrayBuffer_Ext(self.context.inner, self.value) }) != 0
    }

    /// Checks if the JavaScript value is undefined.
    pub fn is_undefined(&self) -> bool {
        self.get_tag() == JS_TAG_UNDEFINED
    }

    /// Checks if the JavaScript value is null.
    pub fn is_null(&self) -> bool {
        self.get_tag() == JS_TAG_NULL
    }

    /// Checks if the JavaScript value is either null or undefined.
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() | self.is_undefined()
    }

    /// Checks if the JavaScript value is a function.
    pub fn is_function(&self) -> bool {
        unsafe { JS_IsFunction(self.context.inner, self.value) != 0 }
    }

    /// Retrieves the value of a property with the specified `key` from the JavaScript object.
    pub fn get_property(&self, key: impl Into<Vec<u8>>) -> Result<Self> {
        let cstring_key = CString::new(key)?;
        let raw =
            unsafe { JS_GetPropertyStr(self.context.inner, self.value, cstring_key.as_ptr()) };
        Self::new(self.context, raw)
    }

    /// Sets the value of a property with the specified `key` on the JavaScript object to `val`.
    pub fn set_property(&self, key: impl Into<Vec<u8>>, val: JSValueRef) -> Result<()> {
        let cstring_key = CString::new(key)?;
        let ret = unsafe {
            JS_DefinePropertyValueStr(
                self.context.inner,
                self.value,
                cstring_key.as_ptr(),
                val.value,
                JS_PROP_C_W_E as i32,
            )
        };

        if ret < 0 {
            let exception = self.as_exception()?;
            return Err(exception.into_error());
        }
        Ok(())
    }

    /// Retrieves the value of an indexed property from the JavaScript object.
    /// This is used for arrays.
    pub fn get_indexed_property(&self, index: u32) -> Result<Self> {
        let raw = unsafe { JS_GetPropertyUint32(self.context.inner, self.value, index) };
        Self::new(self.context, raw)
    }

    /// Appends a property with the value `val` to the JavaScript object.
    /// This is used for arrays.
    pub fn append_property(&self, val: JSValueRef) -> Result<()> {
        let len = self.get_property("length")?;
        let ret = unsafe {
            JS_DefinePropertyValueUint32(
                self.context.inner,
                self.value,
                len.value as u32,
                val.value,
                JS_PROP_C_W_E as i32,
            )
        };

        if ret < 0 {
            let exception = self.as_exception()?;
            return Err(exception.into_error());
        }
        Ok(())
    }

    /// Checks if the JavaScript value is an exception.
    pub fn is_exception(&self) -> bool {
        self.get_tag() == JS_TAG_EXCEPTION
    }

    pub(crate) fn get_tag(&self) -> i32 {
        (self.value >> 32) as i32
    }

    /// All methods in quickjs return an exception value, not an object.
    /// To actually retrieve the exception, we need to retrieve the exception object from the global state.
    fn as_exception(&self) -> Result<Exception> {
        Exception::new(self.context)
    }

    /// Convert the `JSValueRef` to a Rust `JSValue` type.
    fn to_js_value(self) -> Result<JSValue> {
        from_qjs_value(self)
    }
}

// We can't implement From<JSValueRef> for JSValueRaw, as
// JSValueRaw is automatically generated and it would result
// in a cyclic crate dependency.
#[allow(clippy::from_over_into)]
impl Into<JSValueRaw> for JSValueRef<'_> {
    fn into(self) -> JSValueRaw {
        self.value
    }
}

impl fmt::Display for JSValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_js_value().unwrap())
    }
}

/// A macro to implement the `TryFrom<&JSValueRef>` and `TryFrom<JSValueRef>` trait
/// for various Rust types.
macro_rules! try_from_impl {
    ($($t:ty),+ $(,)?) => {
        $(impl TryFrom<&JSValueRef<'_>> for $t {
            type Error = anyhow::Error;

            fn try_from(value: &JSValueRef) -> Result<Self> {
                value.to_js_value()?.try_into()
            }
        }

        impl TryFrom<JSValueRef<'_>> for $t {
            type Error = anyhow::Error;

            fn try_from(value: JSValueRef) -> Result<Self> {
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
    std::collections::HashMap<String, JSValue>,
);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::js_binding::constants::MAX_SAFE_INTEGER;
    use crate::js_binding::constants::MIN_SAFE_INTEGER;

    use super::BigInt;
    use super::{JSContextRef, JSValue};
    use anyhow::Result;
    const SCRIPT_NAME: &str = "value.js";

    #[test]
    fn test_value_objects_allow_retrieving_a_str_property() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "globalThis.bar = 1;";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let prop = global.get_property("bar");
        assert!(prop.is_ok());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_a_str_property() -> Result<()> {
        let ctx = JSContextRef::default();
        let obj = ctx.object_value()?;
        obj.set_property("foo", ctx.value_from_i32(1_i32)?)?;
        let val = obj.get_property("foo");
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_an_indexed_property() {
        let ctx = JSContextRef::default();
        let seq = ctx.array_value().unwrap();
        seq.append_property(ctx.value_from_str("hello").unwrap())
            .unwrap();
        seq.append_property(ctx.value_from_str("world").unwrap())
            .unwrap();

        let val = seq.get_indexed_property(0).unwrap();
        assert_eq!("hello", val.as_str().unwrap());

        let val = seq.get_indexed_property(1).unwrap();
        assert_eq!("world", val.as_str().unwrap());
    }

    #[test]
    fn test_value_set_property_returns_exception() {
        let ctx = JSContextRef::default();
        let val = ctx.value_from_i32(1337).unwrap();
        let err = val
            .set_property("foo", ctx.value_from_str("hello").unwrap())
            .unwrap_err();
        assert_eq!(
            "Uncaught TypeError: not an object\n".to_string(),
            err.to_string()
        );
    }

    #[test]
    fn test_value_append_property_returns_exception() {
        let ctx = JSContextRef::default();
        let val = ctx.value_from_i32(1337).unwrap();
        let err = val
            .append_property(ctx.value_from_str("hello").unwrap())
            .unwrap_err();
        assert_eq!(
            "Uncaught TypeError: not an object\n".to_string(),
            err.to_string()
        );
    }

    #[test]
    fn test_value_objects_allow_retrieving_a_indexed_property() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "globalThis.arr = [1];";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let val = ctx
            .global_object()?
            .get_property("arr")?
            .get_indexed_property(0);
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_allows_representing_a_value_as_f64() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = ctx.value_from_f64(f64::MIN)?.as_f64_unchecked();
        assert_eq!(val, f64::MIN);
        Ok(())
    }

    #[test]
    fn test_value_as_str() {
        let s = "hello";
        let ctx = JSContextRef::default();
        let val = ctx.value_from_str(s).unwrap();
        assert_eq!(val.as_str().unwrap(), s);
    }

    #[test]
    fn test_value_as_str_middle_nul_terminator() {
        let s = "hello\0world!";
        let ctx = JSContextRef::default();
        let val = ctx.value_from_str(s).unwrap();
        assert_eq!(val.as_str().unwrap(), s);
    }

    #[test]
    fn test_exception() {
        let ctx = JSContextRef::default();
        let error = ctx
            .eval_global("main", "should_throw")
            .unwrap_err()
            .to_string();
        let expected_error =
            "Uncaught ReferenceError: \'should_throw\' is not defined\n    at <eval> (main)\n";
        assert_eq!(expected_error, error.as_str());
    }

    #[test]
    fn test_exception_with_stack() {
        let ctx = JSContextRef::default();
        let script = r#"
            function foo() { return bar(); }
            function bar() { return foobar(); }
            function foobar() {
                for (var i = 0; i < 100; i++) {
                    if (i > 25) {
                        throw new Error("boom");
                    }
                }
            }
            foo();
        "#;
        let expected_error = r#"Uncaught Error: boom
    at foobar (main:7)
    at bar (main)
    at foo (main)
    at <eval> (main:11)
"#;
        let error = ctx.eval_global("main", script).unwrap_err().to_string();
        assert_eq!(expected_error, error.as_str());
    }

    #[test]
    fn test_syntax_error() {
        let ctx = JSContextRef::default();
        let error = ctx
            .eval_global("main", "func boom() {}")
            .unwrap_err()
            .to_string();
        let expected_error = "Uncaught SyntaxError: expecting \';\'\n    at main:1\n";
        assert_eq!(expected_error, error.as_str());
    }

    #[test]
    fn test_is_null_or_undefined() {
        let ctx = JSContextRef::default();
        let v = ctx.undefined_value().unwrap();
        assert!(v.is_null_or_undefined());

        let v = ctx.null_value().unwrap();
        assert!(v.is_null_or_undefined());

        let v = ctx.value_from_i32(1337).unwrap();
        assert!(!v.is_null_or_undefined());
    }

    #[test]
    fn test_i64() {
        let ctx = JSContextRef::default();

        // max
        let val = i64::MAX;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(v.is_big_int());
        assert!(!v.is_number());
        assert_eq!(
            BigInt::Unsigned(val as u64),
            v.as_big_int_unchecked().unwrap()
        );

        // min
        let val = i64::MIN;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(v.is_big_int());
        assert!(!v.is_number());
        assert_eq!(BigInt::Signed(val), v.as_big_int_unchecked().unwrap());

        // zero
        let val = 0;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(!v.is_big_int());
        assert!(v.is_number());
        assert_eq!(val, v.as_i32_unchecked() as i64);

        // MAX_SAFE_INTEGER
        let val = MAX_SAFE_INTEGER;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(!v.is_big_int());
        assert!(v.is_number());
        assert_eq!(val, v.as_f64_unchecked() as i64);

        // MAX_SAFE_INTGER + 1
        let val = MAX_SAFE_INTEGER + 1;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(v.is_big_int());
        assert!(!v.is_number());
        assert_eq!(
            BigInt::Unsigned(val as u64),
            v.as_big_int_unchecked().unwrap()
        );

        // MIN_SAFE_INTEGER
        let val = MIN_SAFE_INTEGER;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(!v.is_big_int());
        assert!(v.is_number());
        assert_eq!(val, v.as_f64_unchecked() as i64);

        // MIN_SAFE_INTEGER - 1
        let val = MIN_SAFE_INTEGER - 1;
        let v = ctx.value_from_i64(val).unwrap();
        assert!(v.is_big_int());
        assert!(!v.is_number());
        assert_eq!(BigInt::Signed(val), v.as_big_int_unchecked().unwrap());
    }

    #[test]
    fn test_u64() {
        let ctx = JSContextRef::default();

        // max
        let val = u64::MAX;
        let v = ctx.value_from_u64(val).unwrap();
        assert!(v.is_big_int());
        assert!(!v.is_number());
        assert_eq!(BigInt::Unsigned(val), v.as_big_int_unchecked().unwrap());

        // min == 0
        let val = u64::MIN;
        let v = ctx.value_from_u64(val).unwrap();
        assert!(!v.is_big_int());
        assert!(v.is_number());
        assert_eq!(val, v.as_i32_unchecked() as u64);

        // MAX_SAFE_INTEGER
        let val = MAX_SAFE_INTEGER as u64;
        let v = ctx.value_from_u64(val).unwrap();
        assert!(!v.is_big_int());
        assert!(v.is_number());
        assert_eq!(val, v.as_f64_unchecked() as u64);

        // MAX_SAFE_INTEGER + 1
        let val = (MAX_SAFE_INTEGER + 1) as u64;
        let v = ctx.value_from_u64(val).unwrap();
        assert!(v.is_big_int());
        assert!(!v.is_number());
        assert_eq!(BigInt::Unsigned(val), v.as_big_int_unchecked().unwrap());
    }

    #[test]
    fn test_value_larger_than_u64_max_returns_overflow_error() {
        let ctx = JSContextRef::default();

        ctx.eval_global("main", "var num = BigInt(\"18446744073709551616\");")
            .unwrap(); // u64::MAX + 1
        let num = ctx.global_object().unwrap().get_property("num").unwrap();

        assert!(num.is_big_int());
        assert_eq!(
            "big int overflow, value does not fit in u64",
            num.as_big_int_unchecked().unwrap_err().to_string()
        );
    }

    #[test]
    fn test_value_smaller_than_i64_min_returns_underflow_error() {
        let ctx = JSContextRef::default();

        ctx.eval_global("main", "var num = BigInt(\"-9223372036854775809\");")
            .unwrap(); // i64::MIN - 1
        let num = ctx.global_object().unwrap().get_property("num").unwrap();

        assert!(num.is_big_int());
        assert_eq!(
            "big int underflow, value does not fit in i64",
            num.as_big_int_unchecked().unwrap_err().to_string()
        );
    }

    #[test]
    fn test_u64_creates_an_unsigned_bigint() {
        let ctx = JSContextRef::default();

        let expected = i64::MAX as u64 + 2;
        let v = ctx.value_from_u64(expected).unwrap();

        assert!(v.is_big_int());
        assert_eq!(
            BigInt::Unsigned(expected),
            v.as_big_int_unchecked().unwrap()
        );
    }

    #[test]
    fn test_is_function() {
        let ctx = JSContextRef::default();

        ctx.eval_global("main", "var x = 42; function foo() {}")
            .unwrap();

        assert!(!ctx
            .global_object()
            .unwrap()
            .get_property("x")
            .unwrap()
            .is_function());

        assert!(ctx
            .global_object()
            .unwrap()
            .get_property("foo")
            .unwrap()
            .is_function());
    }

    #[test]
    fn test_eval_function() {
        let ctx = JSContextRef::default();

        let bytecode = ctx.compile_global("main", "var f = 42;").unwrap();
        ctx.value_from_bytecode(&bytecode)
            .unwrap()
            .eval_function()
            .unwrap();

        assert_eq!(
            42,
            ctx.global_object()
                .unwrap()
                .get_property("f")
                .unwrap()
                .try_as_integer()
                .unwrap()
        );
    }

    #[test]
    fn test_convert_bool() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "true")?;

        assert_eq!("true", val.to_string());
        let arg: bool = val.try_into()?;
        assert!(arg);

        let val_ref = &val;
        let arg: bool = val_ref.try_into()?;
        assert!(arg);
        Ok(())
    }

    #[test]
    fn test_convert_i32() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42")?;

        assert_eq!("42", val.to_string());
        let arg: i32 = val.try_into()?;
        assert_eq!(42, arg);

        let val_ref = &val;
        let arg: i32 = val_ref.try_into()?;
        assert_eq!(42, arg);
        Ok(())
    }

    #[test]
    fn test_convert_usize() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42")?;

        assert_eq!("42", val.to_string());
        let arg: usize = val.try_into()?;
        assert_eq!(42, arg);

        let val_ref = &val;
        let arg: usize = val_ref.try_into()?;
        assert_eq!(42, arg);
        Ok(())
    }

    #[test]
    fn test_convert_f64() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "42.42")?;

        assert_eq!("42.42", val.to_string());
        let arg: f64 = val.try_into()?;
        assert_eq!(42.42, arg);

        let val_ref = &val;
        let arg: f64 = val_ref.try_into()?;
        assert_eq!(42.42, arg);
        Ok(())
    }

    #[test]
    fn test_convert_string() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "const h = 'hello'; h")?;

        assert_eq!("hello", val.to_string());
        let arg: String = val.try_into()?;
        assert_eq!("hello", arg);

        let val_ref = &val;
        let arg: String = val_ref.try_into()?;
        assert_eq!("hello", arg);
        Ok(())
    }

    #[test]
    fn test_convert_vec() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "[1, 2, 3]")?;

        let expected: Vec<JSValue> = vec![1.into(), 2.into(), 3.into()];

        assert_eq!("1,2,3", val.to_string());
        let arg: Vec<JSValue> = val.try_into()?;
        assert_eq!(expected, arg);

        let val_ref = &val;
        let arg: Vec<JSValue> = val_ref.try_into()?;
        assert_eq!(expected, arg);
        Ok(())
    }

    #[test]
    fn test_convert_bytes() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "new ArrayBuffer(8)")?;

        let expected = [0_u8; 8].to_vec();

        assert_eq!("[object ArrayBuffer]", val.to_string());
        let arg: Vec<u8> = val.try_into()?;
        assert_eq!(expected, arg);

        let val_ref = &val;
        let arg: Vec<u8> = val_ref.try_into()?;
        assert_eq!(expected, arg);
        Ok(())
    }

    #[test]
    fn test_convert_hashmap() -> Result<()> {
        let context = JSContextRef::default();
        let val = context.eval_global("test.js", "({a: 1, b: 2, c: 3})")?;

        let expected = HashMap::from([
            ("a".to_string(), 1.into()),
            ("b".to_string(), 2.into()),
            ("c".to_string(), 3.into()),
        ]);

        assert_eq!("[object Object]", val.to_string());
        let arg: HashMap<String, JSValue> = val.try_into()?;
        assert_eq!(expected, arg);

        let val_ref = &val;
        let arg: HashMap<String, JSValue> = val_ref.try_into()?;
        assert_eq!(expected, arg);
        Ok(())
    }
}
