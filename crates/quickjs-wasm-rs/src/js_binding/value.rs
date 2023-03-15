use super::context::{Context, CLASSES};
use super::exception::Exception;
use super::properties::Properties;
use anyhow::{anyhow, Result};
use quickjs_wasm_sys::{
    size_t as JS_size_t, JSValue, JS_BigIntSigned, JS_BigIntToInt64, JS_BigIntToUint64, JS_Call,
    JS_DefinePropertyValueStr, JS_DefinePropertyValueUint32, JS_EvalFunction, JS_GetArrayBuffer,
    JS_GetOpaque, JS_GetPropertyStr, JS_GetPropertyUint32, JS_IsArray, JS_IsArrayBuffer_Ext,
    JS_IsFloat64_Ext, JS_IsFunction, JS_ToCStringLen2, JS_ToFloat64, JS_PROP_C_W_E, JS_TAG_BIG_INT,
    JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_INT, JS_TAG_NULL, JS_TAG_OBJECT, JS_TAG_STRING,
    JS_TAG_UNDEFINED,
};
use std::any::TypeId;
use std::borrow::Cow;
use std::cell::RefCell;
use std::ffi::CString;
use std::str;

#[derive(Debug, PartialEq, Eq)]
pub enum BigInt {
    Signed(i64),
    Unsigned(u64),
}

#[derive(Debug, Clone)]
pub struct Value<'a> {
    pub(super) context: &'a Context,
    pub(super) value: JSValue,
}

impl<'a> Value<'a> {
    pub(super) fn new(context: &'a Context, raw_value: JSValue) -> Result<Self> {
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

    pub(super) fn new_unchecked(context: &'a Context, value: JSValue) -> Self {
        Self { context, value }
    }

    pub(super) fn eval_function(&self) -> Result<Self> {
        Self::new(self.context, unsafe {
            JS_EvalFunction(self.context.inner, self.value)
        })
    }

    pub fn call(&self, receiver: &Self, args: &[Self]) -> Result<Self> {
        let args: Vec<JSValue> = args.iter().map(|v| v.value).collect();
        let return_val = unsafe {
            JS_Call(
                self.context.inner,
                self.value,
                receiver.value,
                args.len() as i32,
                args.as_slice().as_ptr() as *mut JSValue,
            )
        };

        Self::new(self.context, return_val)
    }

    pub fn as_i32_unchecked(&self) -> i32 {
        self.value as i32
    }

    pub fn as_u32_unchecked(&self) -> u32 {
        self.value as u32
    }

    pub fn as_f64_unchecked(&self) -> f64 {
        let mut ret = 0_f64;
        unsafe { JS_ToFloat64(self.context.inner, &mut ret, self.value) };
        ret
    }

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

    pub fn is_number(&self) -> bool {
        self.is_repr_as_f64() || self.is_repr_as_i32()
    }

    pub fn is_big_int(&self) -> bool {
        self.get_tag() == JS_TAG_BIG_INT
    }

    pub fn as_f64(&self) -> Result<f64> {
        if self.is_repr_as_f64() {
            return Ok(self.as_f64_unchecked());
        }
        if self.is_repr_as_i32() {
            return Ok(self.as_i32_unchecked() as f64);
        }
        anyhow::bail!("Value is not a number")
    }

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

    pub fn as_bool(&self) -> Result<bool> {
        if self.is_bool() {
            Ok(self.value as i32 > 0)
        } else {
            Err(anyhow!("Can't represent {:?} as bool", self.value))
        }
    }

    pub fn as_str(&self) -> Result<&str> {
        let buffer = self.as_wtf8_str_buffer();
        str::from_utf8(buffer).map_err(Into::into)
    }

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

    pub fn properties(&self) -> Result<Properties> {
        Properties::new(self.context, self.value)
    }

    pub fn is_repr_as_f64(&self) -> bool {
        unsafe { JS_IsFloat64_Ext(self.get_tag()) == 1 }
    }

    pub fn is_repr_as_i32(&self) -> bool {
        self.get_tag() == JS_TAG_INT
    }

    pub fn is_str(&self) -> bool {
        self.get_tag() == JS_TAG_STRING
    }

    pub fn is_bool(&self) -> bool {
        self.get_tag() == JS_TAG_BOOL
    }

    pub fn is_array(&self) -> bool {
        unsafe { JS_IsArray(self.context.inner, self.value) == 1 }
    }

    pub fn is_object(&self) -> bool {
        !self.is_array() && self.get_tag() == JS_TAG_OBJECT
    }

    pub fn is_array_buffer(&self) -> bool {
        (unsafe { JS_IsArrayBuffer_Ext(self.context.inner, self.value) }) != 0
    }

    pub fn is_undefined(&self) -> bool {
        self.get_tag() == JS_TAG_UNDEFINED
    }

    pub fn is_null(&self) -> bool {
        self.get_tag() == JS_TAG_NULL
    }

    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() | self.is_undefined()
    }

    pub fn is_function(&self) -> bool {
        unsafe { JS_IsFunction(self.context.inner, self.value) != 0 }
    }

    pub fn get_property(&self, key: impl Into<Vec<u8>>) -> Result<Self> {
        let cstring_key = CString::new(key)?;
        let raw =
            unsafe { JS_GetPropertyStr(self.context.inner, self.value, cstring_key.as_ptr()) };
        Self::new(self.context, raw)
    }

    pub fn set_property(&self, key: impl Into<Vec<u8>>, val: Value) -> Result<()> {
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

    pub fn get_indexed_property(&self, index: u32) -> Result<Self> {
        let raw = unsafe { JS_GetPropertyUint32(self.context.inner, self.value, index) };
        Self::new(self.context, raw)
    }

    pub fn append_property(&self, val: Value) -> Result<()> {
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

    pub fn is_exception(&self) -> bool {
        self.get_tag() == JS_TAG_EXCEPTION
    }

    /// Get a pointer to the `RefCell` holding a value wrapped using [crate::Context::wrap_rust_value].
    pub fn get_rust_value<T: 'static>(raw: JSValue) -> Result<&'static RefCell<T>> {
        unsafe {
            let pointer = JS_GetOpaque(
                raw,
                *CLASSES
                    .lock()
                    .unwrap()
                    .get(&TypeId::of::<T>())
                    .unwrap(),
            ) as *const RefCell<T>;

            if pointer.is_null() {
                Err(anyhow!("type mismatch"))
            } else {
                Ok(&*pointer)
            }
        }
    }

    fn get_tag(&self) -> i32 {
        (self.value >> 32) as i32
    }

    /// All methods in quickjs return an exception value, not an object.
    /// To actually retrieve the exception, we need to retrieve the exception object from the global state.
    fn as_exception(&self) -> Result<Exception> {
        Exception::new(self.context)
    }
}

// We can't implement From<Value> for JSValue, as
// JSValue is automatically generated and it would result
// in a cyclic crate dependency.
#[allow(clippy::from_over_into)]
impl Into<JSValue> for Value<'_> {
    fn into(self) -> JSValue {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::js_binding::constants::MAX_SAFE_INTEGER;
    use crate::js_binding::constants::MIN_SAFE_INTEGER;

    use super::super::context::Context;
    use super::BigInt;
    use anyhow::Result;
    const SCRIPT_NAME: &str = "value.js";

    #[test]
    fn test_value_objects_allow_retrieving_a_str_property() -> Result<()> {
        let ctx = Context::default();
        let contents = "globalThis.bar = 1;";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let prop = global.get_property("bar");
        assert!(prop.is_ok());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_a_str_property() -> Result<()> {
        let ctx = Context::default();
        let obj = ctx.object_value()?;
        obj.set_property("foo", ctx.value_from_i32(1_i32)?)?;
        let val = obj.get_property("foo");
        assert!(val.is_ok());
        assert!(val.unwrap().is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_value_objects_allow_setting_an_indexed_property() {
        let ctx = Context::default();
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
        let ctx = Context::default();
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
        let ctx = Context::default();
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
        let ctx = Context::default();
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
        let ctx = Context::default();
        let val = ctx.value_from_f64(f64::MIN)?.as_f64_unchecked();
        assert_eq!(val, f64::MIN);
        Ok(())
    }

    #[test]
    fn test_value_as_str() {
        let s = "hello";
        let ctx = Context::default();
        let val = ctx.value_from_str(s).unwrap();
        assert_eq!(val.as_str().unwrap(), s);
    }

    #[test]
    fn test_value_as_str_middle_nul_terminator() {
        let s = "hello\0world!";
        let ctx = Context::default();
        let val = ctx.value_from_str(s).unwrap();
        assert_eq!(val.as_str().unwrap(), s);
    }

    #[test]
    fn test_exception() {
        let ctx = Context::default();
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
        let ctx = Context::default();
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
        let ctx = Context::default();
        let error = ctx
            .eval_global("main", "func boom() {}")
            .unwrap_err()
            .to_string();
        let expected_error = "Uncaught SyntaxError: expecting \';\'\n    at main:1\n";
        assert_eq!(expected_error, error.as_str());
    }

    #[test]
    fn test_is_null_or_undefined() {
        let ctx = Context::default();
        let v = ctx.undefined_value().unwrap();
        assert!(v.is_null_or_undefined());

        let v = ctx.null_value().unwrap();
        assert!(v.is_null_or_undefined());

        let v = ctx.value_from_i32(1337).unwrap();
        assert!(!v.is_null_or_undefined());
    }

    #[test]
    fn test_i64() {
        let ctx = Context::default();

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
        let ctx = Context::default();

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
        let ctx = Context::default();

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
        let ctx = Context::default();

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
        let ctx = Context::default();

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
        let ctx = Context::default();

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
        let ctx = Context::default();

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
}
