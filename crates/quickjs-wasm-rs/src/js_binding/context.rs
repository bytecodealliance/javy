use super::constants::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER};
use super::exception::Exception;
use super::value::Value;
use anyhow::Result;
use once_cell::sync::Lazy;
use quickjs_wasm_sys::{
    ext_js_exception, ext_js_null, ext_js_undefined, size_t as JS_size_t, JSCFunctionData,
    JSClassDef, JSClassID, JSContext, JSValue, JS_Eval, JS_ExecutePendingJob, JS_FreeCString,
    JS_GetGlobalObject, JS_GetRuntime, JS_NewArray, JS_NewArrayBufferCopy, JS_NewBigInt64,
    JS_NewBool_Ext, JS_NewCFunctionData, JS_NewClass, JS_NewClassID, JS_NewContext,
    JS_NewFloat64_Ext, JS_NewInt32_Ext, JS_NewInt64_Ext, JS_NewObject, JS_NewObjectClass,
    JS_NewRuntime, JS_NewStringLen, JS_NewUint32_Ext, JS_SetOpaque, JS_ThrowInternalError,
    JS_ToCStringLen2, JS_EVAL_TYPE_GLOBAL,
};
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Mutex;

pub(super) static CLASSES: Lazy<Mutex<HashMap<TypeId, JSClassID>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

    pub fn execute_pending(&self) -> Result<()> {
        let runtime = unsafe { JS_GetRuntime(self.inner) };

        loop {
            let mut ctx = ptr::null_mut();
            match unsafe { JS_ExecutePendingJob(runtime, &mut ctx) } {
                0 => break Ok(()),
                1 => (),
                _ => break Err(Exception::new(self.inner)?.into_error()),
            }
        }
    }

    pub fn global_object(&self) -> Result<Value> {
        let raw = unsafe { JS_GetGlobalObject(self.inner) };
        Value::new(self.inner, raw)
    }

    pub fn array_value(&self) -> Result<Value> {
        let raw = unsafe { JS_NewArray(self.inner) };
        Value::new(self.inner, raw)
    }

    pub fn array_buffer_value(&self, bytes: &[u8]) -> Result<Value> {
        Value::new(self.inner, unsafe {
            JS_NewArrayBufferCopy(self.inner, bytes.as_ptr(), bytes.len() as _)
        })
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

    /// Get the JS class ID used to wrap instances of the specified Rust type, or else create one if it doesn't
    /// already exist.
    fn get_class_id<T: 'static>(&self) -> JSClassID {
        // Since there is no way (as of this writing) to free a `JSValue` wrapped in a `Value` (i.e. they always
        // leak), there is no need to define a finalizer.  If that changes in the future, this is what the
        // finalizer might look like:
        //
        // ```
        // unsafe extern "C" fn finalize<T: 'static>(_runtime: *mut JSRuntime, value: JSValue) {
        //     let pointer = JS_GetOpaque(
        //         value,
        //         *CLASSES.lock().unwrap().get(&TypeId::of::<T>()).unwrap(),
        //     ) as *mut RefCell<T>;

        //     assert!(!pointer.is_null());

        //     drop(Box::from_raw(pointer))
        // }
        // ```

        *CLASSES
            .lock()
            .unwrap()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| unsafe {
                let mut id = 0;
                JS_NewClassID(&mut id);

                assert!(
                    0 == JS_NewClass(
                        JS_GetRuntime(self.inner),
                        id,
                        &JSClassDef {
                            class_name: b"<rust closure>\0" as *const _ as *const i8,
                            finalizer: None,
                            gc_mark: None,
                            call: None,
                            exotic: ptr::null_mut(),
                        },
                    )
                );

                id
            })
    }

    /// Wrap the specified Rust value in a JS value
    ///
    /// You can use [Value::get_rust_value] to retrieve the original value.
    pub fn wrap_rust_value<T: 'static>(&self, value: T) -> Result<Value> {
        // Note the use of `RefCell` to provide checked unique references.  Since JS values can be arbitrarily
        // aliased, we need `RefCell`'s dynamic borrow checking to prevent unsound access.
        let pointer = Box::into_raw(Box::new(RefCell::new(value)));

        let value = Value::new(self.inner, unsafe {
            JS_NewObjectClass(self.inner, self.get_class_id::<T>().try_into().unwrap())
        })?;

        unsafe {
            JS_SetOpaque(value.value, pointer as *mut c_void);
        }

        Ok(value)
    }

    /// Wrap the specified function in a JS function.
    ///
    /// Since the callback signature accepts parameters as high-level `Context` and `Value` objects, it can be
    /// implemented without using `unsafe` code, unlike [new_callback] which provides a low-level API.
    pub fn wrap_callback<F>(&self, mut f: F) -> Result<Value>
    where
        F: (FnMut(&Self, &Value, &[Value]) -> Result<Value>) + 'static,
    {
        let wrapped = move |inner, this, argc, argv: *mut JSValue, _| match f(
            &Self { inner },
            &Value::new_unchecked(inner, this),
            &(0..argc)
                .map(|offset| Value::new_unchecked(inner, unsafe { *argv.offset(offset as isize) }))
                .collect::<Box<[_]>>(),
        ) {
            Ok(value) => value.value,
            Err(error) => {
                if let Ok(message) = CString::new(format!("{error:?}")) {
                    let format = CString::new("%s").unwrap();
                    unsafe { JS_ThrowInternalError(inner, format.as_ptr(), message.as_ptr()) }
                } else {
                    unsafe { ext_js_exception }
                }
            }
        };

        self.new_callback(wrapped)
    }

    /// Wrap the specified function in a JS function.
    ///
    /// See also [wrap_callback] for a high-level equivalent.
    pub fn new_callback<F>(&self, f: F) -> Result<Value>
    where
        F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue + 'static,
    {
        let trampoline = build_trampoline(&f);

        let mut object = self.wrap_rust_value(f)?;

        let raw =
            unsafe { JS_NewCFunctionData(self.inner, trampoline, 0, 1, 1, &mut object.value) };

        Value::new(self.inner, raw)
    }

    pub fn register_globals<T1, T2>(&mut self, log_stream: T1, error_stream: T2) -> Result<()>
    where
        T1: Write + 'static,
        T2: Write + 'static,
    {
        let console_log_callback = self.new_callback(console_log_to(log_stream))?;
        let console_error_callback = self.new_callback(console_log_to(error_stream))?;
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
) -> impl FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue + 'static
where
    T: Write + 'static,
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
    F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue + 'static,
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
        F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue + 'static,
    {
        (Value::new_unchecked(ctx, *data)
            .get_rust_value::<F>()
            .unwrap()
            .borrow_mut())(ctx, this, argc, argv, magic)
    }

    Some(trampoline::<F>)
}

#[cfg(test)]
mod tests {
    use super::Context;
    use anyhow::Result;
    use quickjs_wasm_sys::ext_js_undefined;
    use std::cell::{Cell, RefCell};
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

    /// This tests that `Context::new_callback` can handle large (i.e. more than a few machine words) closures
    /// correctly.
    #[test]
    fn test_closure() -> Result<()> {
        let ctx = Context::default();

        let global = ctx.global_object()?;

        const LENGTH: usize = 256;
        let array = [42_u8; LENGTH];
        let called = Rc::new(Cell::new(false));

        global.set_property(
            "foo",
            ctx.new_callback({
                let called = called.clone();
                move |_, _, _, _, _| {
                    called.set(true);
                    assert!(array.len() == LENGTH);
                    assert!(array.iter().all(|&v| v == 42));
                    unsafe { ext_js_undefined }
                }
            })?,
        )?;

        ctx.eval_global("main", "foo()")?;

        assert!(called.get());

        Ok(())
    }
}
