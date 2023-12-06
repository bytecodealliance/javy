use super::constants::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER};
use super::exception::Exception;
use super::value::JSValueRef;
use crate::js_value;
use anyhow::Result;
use quickjs_wasm_sys::{
    ext_js_null, ext_js_undefined, JSContext, JSRuntime, JSValue, JS_Eval, JS_ExecutePendingJob,
    JS_FreeContext, JS_FreeRuntime, JS_GetGlobalObject, JS_GetRuntime, JS_IsJobPending,
    JS_NewArray, JS_NewArrayBufferCopy, JS_NewBigInt64, JS_NewBool_Ext, JS_NewContext,
    JS_NewFloat64_Ext, JS_NewInt32_Ext, JS_NewInt64_Ext, JS_NewObject, JS_NewRuntime,
    JS_NewStringLen, JS_NewUint32_Ext, JS_ReadObject, JS_WriteObject, JS_EVAL_FLAG_COMPILE_ONLY,
    JS_EVAL_TYPE_GLOBAL, JS_EVAL_TYPE_MODULE, JS_READ_OBJ_BYTECODE, JS_WRITE_OBJ_BYTECODE,
};
use std::convert::TryInto;
use std::ffi::{c_char, c_int, CString};
use std::fmt::Debug;
use std::ptr;
use std::str;

/// `JSContextRef` is a wrapper around a raw pointer to a QuickJS `JSContext`.
///
/// This struct provides a safe interface for interacting with the underlying
/// QuickJS context. It is primarily used for managing the JavaScript execution
/// environment, such as creating and manipulating JavaScript objects, functions,
/// and values.
///
/// # Safety
///
/// The raw pointer to `JSContext` is not exposed publicly, ensuring that
/// the lifetime of the `JSContextRef` does not outlive the lifetime of the
/// QuickJS context it refers to.
///
/// # Example
///
/// ```
/// // Assuming you have a function to create a new QuickJS context
/// let context = JSContextRef::default();
/// context.eval_global("test.js", "1 + 1")?;
/// ```
pub struct JSContextRef {
    inner: *mut JSContext,
}
impl Default for JSContextRef {
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
impl Drop for JSContextRef {
    fn drop(&mut self) {
        unsafe {
            let runtime = self.runtime_raw();
            JS_FreeContext(self.inner);
            JS_FreeRuntime(runtime);
        }
    }
}
impl JSContextRef {
    pub(super) fn runtime_raw(&self) -> *mut JSRuntime {
        unsafe { JS_GetRuntime(self.inner) }
    }

    /// Evaluates JavaScript code in the global scope.
    ///
    /// This method takes JavaScript code as a string and evaluates it in the global scope of the
    /// JavaScript context.
    ///
    /// # Arguments
    ///
    /// * `name`: A string representing the name of the script.
    /// * `contents`: The JavaScript code to be evaluated as a string.
    ///
    /// # Example
    ///
    /// ```
    /// let context = JSContextRef::default();
    /// context.eval_global("test.js", "1 + 1")?;
    /// ```
    pub fn eval_global(&self, name: &str, contents: &str) -> Result<JSValueRef> {
        self.eval(name, contents, EvalType::Global, false)
    }

    /// Evaluates JavaScript code in an ECMAScript module scope.
    ///
    /// This method takes JavaScript code as a string and evaluates it in a
    /// ECMAScript module scope.
    ///
    /// # Arguments
    ///
    /// * `name`: A string representing the name of the module.
    /// * `contents`: The JavaScript code to be evaluated as a string.
    ///
    /// # Example
    ///
    /// ```
    /// let contenxt = JSContextRef::default();
    /// content.eval_module("test.js", "1 + 1")?;
    /// ```
    pub fn eval_module(&self, name: &str, contents: &str) -> Result<JSValueRef> {
        self.eval(name, contents, EvalType::Module, false)
    }

    fn eval(
        &self,
        name: &str,
        contents: &str,
        eval_as: EvalType,
        compile_only: bool,
    ) -> Result<JSValueRef> {
        let input = CString::new(contents)?;
        let script_name = CString::new(name)?;
        let len = contents.len() - 1;
        let mut eval_flags = match eval_as {
            EvalType::Global => JS_EVAL_TYPE_GLOBAL,
            EvalType::Module => JS_EVAL_TYPE_MODULE,
        };
        if compile_only {
            eval_flags |= JS_EVAL_FLAG_COMPILE_ONLY;
        }
        let raw = unsafe {
            JS_Eval(
                self.inner,
                input.as_ptr(),
                len as _,
                script_name.as_ptr(),
                eval_flags as i32,
            )
        };

        JSValueRef::new(self, raw)
    }

    /// Returns the raw pointer to the underlying [quickjs_wasm_sys::JSContext].
    ///
    /// # Safety
    /// While calling this function is safe, using it’s return value has to be
    /// done with a sufficitenly deep understanding of QuickJS and [quickjs_wasm_sys].
    ///
    /// This function is not part of the crate’s semver API contract.
    #[cfg_attr(feature = "export-sys", visibility::make(pub))]
    pub(super) unsafe fn as_raw(&self) -> *mut JSContext {
        self.inner
    }

    /// Creates a `JSContextRef` from a pointer to a [quickjs_wasm_sys::JSContext].
    ///
    /// # Safety
    /// The caller has to ensure that the returned `JSContextRef` is the only one
    /// used throughout its lifetime.
    ///
    /// This function is not part of the crate’s semver API contract.
    #[cfg_attr(feature = "export-sys", visibility::make(pub))]
    pub(super) unsafe fn from_raw(inner: *mut JSContext) -> JSContextRef {
        JSContextRef { inner }
    }

    /// Compiles JavaScript to QuickJS bytecode with an ECMAScript module scope.
    ///
    /// # Arguments
    ///
    /// * `name`: A string representing the name of the script.
    /// * `contents`: The JavaScript code to be compiled as a string.
    pub fn compile_module(&self, name: &str, contents: &str) -> Result<Vec<u8>> {
        self.compile(name, contents, EvalType::Module)
    }

    /// Compiles JavaScript to QuickJS bytecode with a global scope.
    ///
    /// # Arguments
    ///
    /// * `name`: A string representing the name of the script.
    /// * `contents`: The JavaScript code to be compiled as a string.
    pub fn compile_global(&self, name: &str, contents: &str) -> Result<Vec<u8>> {
        self.compile(name, contents, EvalType::Global)
    }

    fn compile(&self, name: &str, contents: &str, compile_as: EvalType) -> Result<Vec<u8>> {
        let raw = self.eval(name, contents, compile_as, true)?;

        let mut output_size = 0;
        unsafe {
            let output_buffer = JS_WriteObject(
                self.inner,
                &mut output_size,
                raw.as_raw(),
                JS_WRITE_OBJ_BYTECODE as i32,
            );
            Ok(Vec::from_raw_parts(
                output_buffer,
                output_size.try_into()?,
                output_size.try_into()?,
            ))
        }
    }

    /// Evaluate QuickJS bytecode produced by [`Self::compile_module`] or
    /// [`Self::compile_global`].
    pub fn eval_binary(&self, bytecode: &[u8]) -> Result<JSValueRef> {
        self.value_from_bytecode(bytecode)?.eval_function()
    }

    /// Checks if there are any pending jobs in the JavaScript context.
    ///
    /// This method returns `true` if there are pending jobs (for example, promises) in the
    /// JavaScript context, and `false` otherwise.
    pub fn is_pending(&self) -> bool {
        unsafe {
            let runtime = JS_GetRuntime(self.inner);
            JS_IsJobPending(runtime) == 1
        }
    }

    /// Executes all pending jobs in the JavaScript context.
    ///
    /// This method executes all pending jobs (e.g., promises) in the JavaScript context
    /// until there are no more pending jobs or an exception occurs. It returns a `Result` indicating
    /// whether the execution was successful or an error if an exception was thrown.
    pub fn execute_pending(&self) -> Result<()> {
        let runtime = unsafe { JS_GetRuntime(self.inner) };

        loop {
            let mut ctx = ptr::null_mut();
            match unsafe { JS_ExecutePendingJob(runtime, &mut ctx) } {
                0 => break Ok(()),
                1 => (),
                _ => break Err(Exception::new(self)?.into_error()),
            }
        }
    }

    /// Retrieves the global object of the JavaScript context.
    pub fn global_object(&self) -> Result<JSValueRef> {
        let raw = unsafe { JS_GetGlobalObject(self.inner) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript Array object.
    pub fn array_value(&self) -> Result<JSValueRef> {
        let raw = unsafe { JS_NewArray(self.inner) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript ArrayBuffer object with the specified bytes.
    pub fn array_buffer_value(&self, bytes: &[u8]) -> Result<JSValueRef> {
        JSValueRef::new(self, unsafe {
            JS_NewArrayBufferCopy(self.inner, bytes.as_ptr(), bytes.len() as _)
        })
    }

    /// Creates a new JavaScript Object.
    pub fn object_value(&self) -> Result<JSValueRef> {
        let raw = unsafe { JS_NewObject(self.inner) };
        JSValueRef::new(self, raw)
    }

    pub(super) fn value_from_bytecode(&self, bytecode: &[u8]) -> Result<JSValueRef> {
        JSValueRef::new(self, unsafe {
            JS_ReadObject(
                self.inner,
                bytecode.as_ptr(),
                bytecode.len().try_into()?,
                JS_READ_OBJ_BYTECODE.try_into()?,
            )
        })
    }

    /// Creates a new JavaScript Number object from a `f64` value.
    pub fn value_from_f64(&self, val: f64) -> Result<JSValueRef> {
        let raw = unsafe { JS_NewFloat64_Ext(self.inner, val) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript Number object from an `i32` value.
    pub fn value_from_i32(&self, val: i32) -> Result<JSValueRef> {
        let raw = unsafe { JS_NewInt32_Ext(self.inner, val) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript Number or BigInt object from an `i64` value.
    pub fn value_from_i64(&self, val: i64) -> Result<JSValueRef> {
        let raw = if (MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&val) {
            unsafe { JS_NewInt64_Ext(self.inner, val) }
        } else {
            unsafe { JS_NewBigInt64(self.inner, val) }
        };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript Number or BigInt object from a `u64` value.
    pub fn value_from_u64(&self, val: u64) -> Result<JSValueRef> {
        if val <= MAX_SAFE_INTEGER as u64 {
            let raw = unsafe { JS_NewInt64_Ext(self.inner, val as i64) };
            JSValueRef::new(self, raw)
        } else {
            let value = self.value_from_str(&val.to_string())?;
            let bigint = self.global_object()?.get_property("BigInt")?;
            bigint.call(&bigint, &[value])
        }
    }

    /// Creates a new JavaScript Number object from a `u32` value.
    pub fn value_from_u32(&self, val: u32) -> Result<JSValueRef> {
        let raw = unsafe { JS_NewUint32_Ext(self.inner, val) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript Boolean object from a `bool` value.
    pub fn value_from_bool(&self, val: bool) -> Result<JSValueRef> {
        let raw = unsafe { JS_NewBool_Ext(self.inner, i32::from(val)) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript String object from a `&str` value.
    pub fn value_from_str(&self, val: &str) -> Result<JSValueRef> {
        let raw =
            unsafe { JS_NewStringLen(self.inner, val.as_ptr() as *const c_char, val.len() as _) };
        JSValueRef::new(self, raw)
    }

    /// Creates a new JavaScript Null object.
    pub fn null_value(&self) -> Result<JSValueRef> {
        JSValueRef::new(self, unsafe { ext_js_null })
    }

    /// Creates a new JavaScript Undefined object.
    pub fn undefined_value(&self) -> Result<JSValueRef> {
        JSValueRef::new(self, unsafe { ext_js_undefined })
    }

    pub fn wrap_callback<F>(&self, f: F) -> Result<JSValueRef>
    where
        F: (FnMut(&Self, &JSValueRef, &[JSValueRef]) -> Result<js_value::JSValue>) + 'static,
    {
        super::callback::wrap(f).into_js_value(self)
    }

    pub fn new_callback<F>(&self, f: F) -> Result<JSValueRef>
    where
        F: FnMut(*mut JSContext, JSValue, c_int, *mut JSValue) -> JSValue + 'static,
    {
        super::callback::Callback::new(f).into_js_value(self)
    }
}
impl Debug for JSContextRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSContextRef({:p})", self.inner)
    }
}

enum EvalType {
    Global,
    Module,
}

#[cfg(test)]
mod tests {
    use super::JSContextRef;
    use anyhow::Result;
    const SCRIPT_NAME: &str = "context.js";

    #[test]
    fn test_new_returns_a_context() -> Result<()> {
        let _ = JSContextRef::default();
        Ok(())
    }

    #[test]
    fn test_context_evalutes_code_globally() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "var a = 1;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_reports_invalid_code() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "a + 1 * z;";
        let val = ctx.eval_global(SCRIPT_NAME, contents);
        assert!(val.is_err());
        Ok(())
    }

    #[test]
    fn test_context_evaluates_code_in_a_module() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "export let a = 1;";
        let val = ctx.eval_module(SCRIPT_NAME, contents);
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_reports_invalid_module_code() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "a + 1 * z;";
        let val = ctx.eval_module(SCRIPT_NAME, contents);
        assert!(val.is_err());
        Ok(())
    }

    #[test]
    fn test_context_allows_access_to_global_object() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = ctx.global_object();
        assert!(val.is_ok());
        Ok(())
    }

    #[test]
    fn test_context_allows_calling_a_function() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "globalThis.foo = function() { return 1; }";
        let _ = ctx.eval_global(SCRIPT_NAME, contents)?;
        let global = ctx.global_object()?;
        let fun = global.get_property("foo")?;
        let result = fun.call(&global, &[]);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_compile_global_compiles_and_evaluates_global_object() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "var foo = 42;";
        let bytecode = ctx.compile_global(SCRIPT_NAME, contents)?;
        let _ = ctx.eval_binary(&bytecode)?;
        assert_eq!(
            42,
            ctx.global_object()?.get_property("foo")?.try_as_integer()?
        );
        Ok(())
    }

    #[test]
    fn test_compile_module_does_not_implicitly_change_global_object() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "var foo = 42;";
        let bytecode = ctx.compile_module(SCRIPT_NAME, contents)?;
        let _ = ctx.eval_binary(&bytecode)?;
        assert!(ctx.global_object()?.get_property("foo")?.is_undefined());
        Ok(())
    }

    #[test]
    fn test_compile_module_bytecode_evaluates() -> Result<()> {
        let ctx = JSContextRef::default();
        ctx.eval_global("foo.js", "globalThis.foo = 1;")?;
        let bytecode = ctx.compile_module(SCRIPT_NAME, "foo += 1;")?;
        let _ = ctx.eval_binary(&bytecode)?;
        assert_eq!(
            2,
            ctx.global_object()?.get_property("foo")?.try_as_integer()?
        );
        Ok(())
    }

    #[test]
    fn test_compile_global_errors_when_invalid_using_syntax() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "import 'foo';";
        let res = ctx.compile_global(SCRIPT_NAME, contents);
        let err = res.unwrap_err();
        assert!(err.to_string().starts_with("Uncaught SyntaxError"));
        Ok(())
    }

    #[test]
    fn test_compile_module_errors_when_importing() -> Result<()> {
        let ctx = JSContextRef::default();
        let contents = "import 'foo';";
        let res = ctx.compile_module(SCRIPT_NAME, contents);
        let err = res.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Uncaught ReferenceError: could not load module 'foo'\n"
        );
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_f64() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = f64::MIN;
        let val = ctx.value_from_f64(val)?;
        assert!(val.is_repr_as_f64());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_i32() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = i32::MIN;
        let val = ctx.value_from_i32(val)?;
        assert!(val.is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_u32() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = u32::MIN;
        let val = ctx.value_from_u32(val)?;
        assert!(val.is_repr_as_i32());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_bool() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = ctx.value_from_bool(false)?;
        assert!(val.is_bool());
        Ok(())
    }

    #[test]
    fn test_creates_a_value_from_str() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = "script.js";
        let val = ctx.value_from_str(val)?;
        assert!(val.is_str());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_array() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = ctx.array_value()?;
        assert!(val.is_array());
        Ok(())
    }

    #[test]
    fn test_constructs_a_value_as_an_object() -> Result<()> {
        let ctx = JSContextRef::default();
        let val = ctx.object_value()?;
        assert!(val.is_object());
        Ok(())
    }

    #[test]
    fn test_is_pending_returns_false_when_nothing_is_pending() -> Result<()> {
        let ctx = JSContextRef::default();
        ctx.eval_global("main", "const x = 42;")?;
        assert!(!ctx.is_pending());
        Ok(())
    }

    #[test]
    fn test_is_pending_returns_true_when_pending() -> Result<()> {
        let ctx = JSContextRef::default();
        ctx.eval_global(
            "main",
            "
            async function foo() {
                const x = 42;
            }
            foo().then(() => {})",
        )?;
        assert!(ctx.is_pending());
        Ok(())
    }

    #[test]
    fn test_json_parse_works() -> Result<()> {
        let ctx = JSContextRef::default();
        ctx.eval_global(
            "main",
            r#"
            function main() {
                const a = JSON.parse('{ "hello": "there", "I": ["welcome", "you"] }');
                if (a.hello != "there" || a.I[0] != "welcome" || a.I[1] != "you") {
                    throw new Error('error')
                }
            }
            main()
            "#,
        )?;
        Ok(())
    }
}
