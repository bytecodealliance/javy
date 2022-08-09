use super::constants::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER};
use super::value::Value;
use anyhow::Result;
use quickjs_wasm_sys::{
    ext_js_exception, ext_js_null, ext_js_undefined, size_t as JS_size_t, JSCFunctionData,
    JSContext, JSValue, JS_Eval, JS_FreeCString, JS_GetGlobalObject, JS_NewArray, JS_NewBigInt64,
    JS_NewBool_Ext, JS_NewCFunctionData, JS_NewContext, JS_NewFloat64_Ext, JS_NewInt32_Ext,
    JS_NewInt64_Ext, JS_NewObject, JS_NewRuntime, JS_NewStringLen, JS_NewUint32_Ext,
    JS_ToCStringLen2, JS_EVAL_TYPE_GLOBAL,
};
use std::ffi::CString;
use std::io::Write;
use std::os::raw::{c_char, c_int, c_void};

#[derive(Debug)]
pub struct Context {
    inner: *mut JSContext,
}

impl Default for Context {
    fn default() -> Self {
        let runtime = unsafe { JS_NewRuntime() };
        if runtime.is_null() {
            panic!("Couldn't create JavaScript runtime");
        }

        let inner = unsafe { JS_NewContext(runtime) };
        if inner.is_null() {
            panic!("Couldn't create JavaScript context");
        }

        Self { inner }
    }
}

impl Context {
    pub fn eval_global(&self, name: &str, contents: &str) -> Result<Value> {
        let input = CString::new(contents)?;
        let script_name = CString::new(name)?;
        let len = contents.len() - 1;
        let raw = unsafe {
            JS_Eval(
                self.inner,
                input.as_ptr(),
                len as _,
                script_name.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        Value::new(self.inner, raw)
    }

    pub fn global_object(&self) -> Result<Value> {
        let raw = unsafe { JS_GetGlobalObject(self.inner) };
        Value::new(self.inner, raw)
    }

    pub fn array_value(&self) -> Result<Value> {
        let raw = unsafe { JS_NewArray(self.inner) };
        Value::new(self.inner, raw)
    }

    pub fn object_value(&self) -> Result<Value> {
        let raw = unsafe { JS_NewObject(self.inner) };
        Value::new(self.inner, raw)
    }

    pub fn value_from_f64(&self, val: f64) -> Result<Value> {
        let raw = unsafe { JS_NewFloat64_Ext(self.inner, val) };
        Value::new(self.inner, raw)
    }

    pub fn value_from_i32(&self, val: i32) -> Result<Value> {
        let raw = unsafe { JS_NewInt32_Ext(self.inner, val) };
        Value::new(self.inner, raw)
    }

    pub fn value_from_i64(&self, val: i64) -> Result<Value> {
        let raw = if (MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&val) {
            unsafe { JS_NewInt64_Ext(self.inner, val) }
        } else {
            unsafe { JS_NewBigInt64(self.inner, val) }
        };
        Value::new(self.inner, raw)
    }

    pub fn value_from_u64(&self, val: u64) -> Result<Value> {
        if val <= MAX_SAFE_INTEGER as u64 {
            let raw = unsafe { JS_NewInt64_Ext(self.inner, val as i64) };
            Value::new(self.inner, raw)
        } else {
            let value = self.value_from_str(&val.to_string())?;
            let bigint = self.global_object()?.get_property("BigInt")?;
            bigint.call(&bigint, &[value])
        }
    }

    pub fn value_from_u32(&self, val: u32) -> Result<Value> {
        let raw = unsafe { JS_NewUint32_Ext(self.inner, val) };
        Value::new(self.inner, raw)
    }

    pub fn value_from_bool(&self, val: bool) -> Result<Value> {
        let raw = unsafe { JS_NewBool_Ext(self.inner, i32::from(val)) };
        Value::new(self.inner, raw)
    }

    pub fn value_from_str(&self, val: &str) -> Result<Value> {
        let raw =
            unsafe { JS_NewStringLen(self.inner, val.as_ptr() as *const c_char, val.len() as _) };
        Value::new(self.inner, raw)
    }

    pub fn null_value(&self) -> Result<Value> {
        Value::new(self.inner, unsafe { ext_js_null })
    }

    pub fn undefined_value(&self) -> Result<Value> {
        Value::new(self.inner, unsafe { ext_js_undefined })
    }

    /// # Safety
    ///
    /// The lifetime of values used in `f` are not respected so you need to
    /// manually ensure any referenced variables live at least as long as this
    /// context.
    ///
    /// The following example will result in undefined behavior:
    ///
    /// ```rs
    /// let bar = "bar".to_string();
    /// self.create_callback(|_, _, _, _, _| println!("foo: {}", &bar));
    /// ```
    pub unsafe fn new_callback<F>(&self, f: F) -> Result<Value>
    where
        F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue,
    {
        let trampoline = build_trampoline(&f);
        let data = &f as *const _ as *mut c_void as *mut JSValue;

        let raw = JS_NewCFunctionData(self.inner, trampoline, 0, 1, 1, data);
        Value::new(self.inner, raw)
    }

    pub fn register_globals<T1, T2>(&mut self, log_stream: T1, error_stream: T2) -> Result<()>
    where
        T1: Write,
        T2: Write,
    {
        let console_log_callback = unsafe { self.new_callback(console_log_to(log_stream))? };
        let console_error_callback = unsafe { self.new_callback(console_log_to(error_stream))? };
        let global_object = self.global_object()?;
        let console_object = self.object_value()?;
        console_object.set_property("log", console_log_callback)?;
        console_object.set_property("error", console_error_callback)?;
        global_object.set_property("console", console_object)?;
        Ok(())
    }
}

fn console_log_to<T>(
    mut stream: T,
) -> impl FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue
where
    T: Write,
{
    move |ctx: *mut JSContext, _this: JSValue, argc: c_int, argv: *mut JSValue, _magic: c_int| {
        let mut len: JS_size_t = 0;
        for i in 0..argc {
            if i != 0 {
                write!(stream, " ").unwrap();
            }

            let str_ptr = unsafe { JS_ToCStringLen2(ctx, &mut len, *argv.offset(i as isize), 0) };
            if str_ptr.is_null() {
                return unsafe { ext_js_exception };
            }

            let str_ptr = str_ptr as *const u8;
            let str_len = len as usize;
            let buffer = unsafe { std::slice::from_raw_parts(str_ptr, str_len) };

            stream.write_all(buffer).unwrap();
            unsafe { JS_FreeCString(ctx, str_ptr as *const i8) };
        }

        writeln!(stream,).unwrap();
        unsafe { ext_js_undefined }
    }
}

fn build_trampoline<F>(_f: &F) -> JSCFunctionData
where
    F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue,
{
    // We build a trampoline to jump between c <-> rust and allow closing over a specific context.
    // For more info around how this works, see https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/.
    unsafe extern "C" fn trampoline<F>(
        ctx: *mut JSContext,
        this: JSValue,
        argc: c_int,
        argv: *mut JSValue,
        magic: c_int,
        data: *mut JSValue,
    ) -> JSValue
    where
        F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue,
    {
        let closure_ptr = data;
        let closure: &mut F = &mut *(closure_ptr as *mut F);
        (*closure)(ctx, this, argc, argv, magic)
    }

    Some(trampoline::<F>)
}

#[cfg(test)]
mod tests {
    use super::Context;
    use anyhow::Result;
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;
    const SCRIPT_NAME: &str = "context.js";

    #[test]
    fn test_new_returns_a_context() -> Result<()> {
        let _ = Context::default();
        Ok(())
    }

    #[test]
    fn test_context_evalutes_code_globally() -> Result<()> {
        let ctx = Context::default();
        let contents = "var a = 1;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_reports_invalid_code() -> Result<()> {
        let ctx = Context::default();
        let contents = "a + 1 * z;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_err());
        Ok(())
    }

    #[test]
    fn test_context_allows_access_to_global_object() -> Result<()> {
        let ctx = Context::default();
        let val = ctx.global_object();
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_allows_calling_a_function() -> Result<()> {
        let ctx = Context::default();
        let contents = "globalThis.foo = function() { return 1; }";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let fun = global.get_property("foo")?;
        let result = fun.call(&global, &[]);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_f64() -> Result<()> {
        let ctx = Context::default();
        let val = f64::MIN;
        let val = ctx.value_from_f64(val)?;
        assert!(val.is_repr_as_f64());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_i32() -> Result<()> {
        let ctx = Context::default();
        let val = i32::MIN;
        let val = ctx.value_from_i32(val)?;
        assert!(val.is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_u32() -> Result<()> {
        let ctx = Context::default();
        let val = u32::MIN;
        let val = ctx.value_from_u32(val)?;
        assert!(val.is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_bool() -> Result<()> {
        let ctx = Context::default();
        let val = ctx.value_from_bool(false)?;
        assert!(val.is_bool());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_str() -> Result<()> {
        let ctx = Context::default();
        let val = "script.js";
        let val = ctx.value_from_str(val)?;
        assert!(val.is_str());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_array() -> Result<()> {
        let ctx = Context::default();
        let val = ctx.array_value()?;
        assert!(val.is_array());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_object() -> Result<()> {
        let val = Context::default().object_value()?;
        assert!(val.is_object());
        Ok(())
    }

    #[derive(Default, Clone)]
    struct SharedStream(Rc<RefCell<Vec<u8>>>);

    impl SharedStream {
        fn clear(&mut self) {
            (*self.0).borrow_mut().clear();
        }
    }

    impl io::Write for SharedStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            (*self.0).borrow_mut().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            (*self.0).borrow_mut().flush()
        }
    }

    #[test]
    fn test_console_log() -> Result<()> {
        let mut stream = SharedStream::default();

        let mut ctx = Context::default();
        ctx.register_globals(stream.clone(), stream.clone())?;

        ctx.eval_global("main", "console.log(\"hello world\");")?;
        assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

        stream.clear();

        ctx.eval_global("main", "console.log(\"bonjour\", \"le\", \"monde\")")?;
        assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());
        Ok(())
    }

    #[test]
    fn test_console_error() -> Result<()> {
        let mut stream = SharedStream::default();

        let mut ctx = Context::default();
        ctx.register_globals(stream.clone(), stream.clone())?;

        ctx.eval_global("main", "console.error(\"hello world\");")?;
        assert_eq!(b"hello world\n", stream.0.borrow().as_slice());

        stream.clear();

        ctx.eval_global("main", "console.error(\"bonjour\", \"le\", \"monde\")")?;
        assert_eq!(b"bonjour le monde\n", stream.0.borrow().as_slice());
        Ok(())
    }
}
