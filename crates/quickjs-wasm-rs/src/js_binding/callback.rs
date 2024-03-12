use std::{
    cell::RefCell,
    ffi::{c_int, c_void, CString},
    ops::{Deref, DerefMut},
};

use once_cell::sync::Lazy;
use quickjs_wasm_sys::{
    JSClassDef, JSClassID, JSContext, JSRuntime, JSValue as JSValueRaw, JS_GetOpaque,
    JS_GetOpaque2, JS_IsRegisteredClass, JS_NewClass, JS_NewClassID, JS_NewObjectClass,
    JS_SetOpaque, JS_ThrowInternalError, JS_ThrowRangeError, JS_ThrowReferenceError,
    JS_ThrowSyntaxError, JS_ThrowTypeError,
};

use crate::{
    js_value::{qjs_convert, JSValue},
    JSContextRef, JSError, JSValueRef,
};

// see <https://docs.rs/rquickjs-core/0.3.1/src/rquickjs_core/class_id.rs.html>
static CALLBACK_CLASS_ID: Lazy<JSClassID> = Lazy::new(|| {
    let mut class_id = 0;
    unsafe {
        JS_NewClassID(&mut class_id);
    }
    class_id
});

type DynCallback =
    dyn FnMut(*mut JSContext, JSValueRaw, c_int, *mut JSValueRaw) -> JSValueRaw + 'static;
/// Represents a Rust callback function that can be called from JavaScript.
pub struct Callback(Box<RefCell<DynCallback>>);
impl Callback {
    /// Custom class id.
    fn class_id() -> JSClassID {
        *CALLBACK_CLASS_ID.deref()
    }

    /// Register the custom class into the `runtime`.
    unsafe fn register(runtime: *mut JSRuntime) {
        // see <https://docs.rs/rquickjs-core/0.3.1/src/rquickjs_core/value/function/ffi.rs.html#53>
        let class_id = Self::class_id();
        if JS_IsRegisteredClass(runtime, class_id) == 0 {
            let class_def = JSClassDef {
                class_name: b"<rust closure>\0" as *const _ as *const i8,
                finalizer: Some(Self::finalizer),
                gc_mark: None,
                call: Some(Self::call),
                exotic: std::ptr::null_mut(),
            };
            unsafe {
                assert_eq!(JS_NewClass(runtime, class_id, &class_def), 0);
            }
        }
    }

    unsafe extern "C" fn call(
        context: *mut JSContext,
        func: JSValueRaw,
        this: JSValueRaw,
        argc: c_int,
        argv: *mut JSValueRaw,
        _flags: c_int,
    ) -> JSValueRaw {
        let boxed_self = unsafe { JS_GetOpaque2(context, func, Self::class_id()) } as *mut Self;
        let self_ref = &*boxed_self;

        // TODO: handle panics in closures, as that can't unwind over ffi

        let mut closure = self_ref.0.borrow_mut();
        (closure.deref_mut())(context, this, argc, argv)
    }

    unsafe extern "C" fn finalizer(_runtime: *mut JSRuntime, val: JSValueRaw) {
        let boxed_self = unsafe { JS_GetOpaque(val, Self::class_id()) as *mut Self };
        std::mem::drop(Box::from_raw(boxed_self));
    }

    /// Wrap the specified function in a JS function.
    ///
    /// See also [wrap] for a high-level equivalent.
    pub fn new<
        F: FnMut(*mut JSContext, JSValueRaw, c_int, *mut JSValueRaw) -> JSValueRaw + 'static,
    >(
        f: F,
    ) -> Self {
        Self(Box::new(RefCell::new(f)))
    }

    pub fn into_js_value(self, context: &JSContextRef) -> anyhow::Result<JSValueRef> {
        let raw = unsafe {
            Self::register(context.runtime_raw());

            // create a new object
            let obj = JS_NewObjectClass(context.as_raw(), Self::class_id() as _);
            let boxed_self = Box::into_raw(Box::new(self));

            // and set its opaque value to a pointer we leak here
            JS_SetOpaque(obj, boxed_self as *mut c_void);

            obj
        };

        JSValueRef::new(context, raw)
    }
}
/// Wrap the specified function in a JS function.
///
/// Since the callback signature accepts parameters as high-level `JSContextRef` and `JSValueRef` objects, it can be
/// implemented without using `unsafe` code, unlike [JSContextRef::new_callback] which provides a low-level API.
/// Returning a [JSError] from the callback will cause a JavaScript error with the appropriate
/// type to be thrown.
pub fn wrap<F>(mut f: F) -> Callback
where
    F: (FnMut(&JSContextRef, &JSValueRef, &[JSValueRef]) -> anyhow::Result<JSValue>) + 'static,
{
    let wrapped = move |inner, this, argc, argv: *mut JSValueRaw| {
        // do not drop this these refs, they are only shared by reference
        let inner_ctx = std::mem::ManuallyDrop::new(unsafe { JSContextRef::from_raw(inner) });
        let this = std::mem::ManuallyDrop::new(unsafe { JSValueRef::from_raw(&inner_ctx, this) });

        // construct args array
        let mut args = Vec::<JSValueRef>::with_capacity(argc as usize);
        for i in 0..argc as isize {
            let value = unsafe {
                JSValueRef::from_raw(
                    &inner_ctx,
                    *argv.offset(i), // SAFETY: within 0..argc
                )
            };
            // these arguments shouldn't be freed at the end of the callback
            // so we can either clone+forget here to increase their reference count, or we could mark the whole Vec as manually drop
            // but then we'd have make sure we don't leak the Vec while not dropping the values, so this looks easier/safer
            //
            // the best way would probably be to have `Vec<ManuallyDrop<T>>` but there aren't any fancy methods in std for turning that into a `&[T]`
            std::mem::forget(value.clone());
            args.push(value);
        }

        let res = match f(&inner_ctx, &this, &args) {
            Ok(value) => {
                let (_, value) = unsafe {
                    qjs_convert::to_qjs_value(&inner_ctx, &value)
                        .unwrap()
                        .into_raw() // do not drop the value, move it into QuickJS
                };

                value
            }
            Err(error) => {
                let format = CString::new("%s").unwrap();
                match error.downcast::<JSError>() {
                    Ok(js_error) => {
                        let message = CString::new(js_error.to_string())
                            .unwrap_or_else(|_| CString::new("Unknown error").unwrap());
                        match js_error {
                            JSError::Internal(_) => unsafe {
                                JS_ThrowInternalError(inner, format.as_ptr(), message.as_ptr())
                            },
                            JSError::Syntax(_) => unsafe {
                                JS_ThrowSyntaxError(inner, format.as_ptr(), message.as_ptr())
                            },
                            JSError::Type(_) => unsafe {
                                JS_ThrowTypeError(inner, format.as_ptr(), message.as_ptr())
                            },
                            JSError::Reference(_) => unsafe {
                                JS_ThrowReferenceError(inner, format.as_ptr(), message.as_ptr())
                            },
                            JSError::Range(_) => unsafe {
                                JS_ThrowRangeError(inner, format.as_ptr(), message.as_ptr())
                            },
                        }
                    }
                    Err(e) => {
                        let message = format!("{e:?}");
                        let message = CString::new(message.as_str()).unwrap_or_else(|err| {
                            CString::new(format!("{} - truncated due to null byte", unsafe {
                                std::str::from_utf8_unchecked(
                                    &message.as_bytes()[..err.nul_position()],
                                )
                            }))
                            .unwrap()
                        });
                        unsafe { JS_ThrowInternalError(inner, format.as_ptr(), message.as_ptr()) }
                    }
                }
            }
        };

        res
    };

    Callback::new(wrapped)
}

#[cfg(test)]
mod test {
    use quickjs_wasm_sys::ext_js_undefined;

    use super::*;
    use std::{cell::Cell, collections::HashMap, rc::Rc};

    /// This tests that `Context::new_callback` can handle large (i.e. more than a few machine words) closures
    /// correctly.
    #[test]
    fn test_closure() -> anyhow::Result<()> {
        let ctx = JSContextRef::default();

        let global = ctx.global_object()?;

        const LENGTH: usize = 256;
        let array = [42_u8; LENGTH];
        let called = Rc::new(Cell::new(false));

        global.set_property(
            "foo",
            Callback::new({
                let called = called.clone();
                move |_, _, _, _| {
                    called.set(true);
                    assert!(array.len() == LENGTH);
                    assert!(array.iter().all(|&v| v == 42));
                    unsafe { ext_js_undefined }
                }
            })
            .into_js_value(&ctx)?,
        )?;

        ctx.eval_global("main", "foo()")?;

        assert!(called.get());

        Ok(())
    }

    #[test]
    fn test_wrap_callback_can_throw_typed_errors() -> anyhow::Result<()> {
        error_test_case(|| JSError::Internal("".to_string()), "InternalError")?;
        error_test_case(|| JSError::Range("".to_string()), "RangeError")?;
        error_test_case(|| JSError::Reference("".to_string()), "ReferenceError")?;
        error_test_case(|| JSError::Syntax("".to_string()), "SyntaxError")?;
        error_test_case(|| JSError::Type("".to_string()), "TypeError")?;
        Ok(())
    }

    fn error_test_case<F>(error: F, js_type: &str) -> anyhow::Result<()>
    where
        F: Fn() -> JSError + 'static,
    {
        let ctx = JSContextRef::default();
        ctx.global_object()?.set_property(
            "foo",
            super::wrap(move |_, _, _| Err(error().into())).into_js_value(&ctx)?,
        )?;
        ctx.eval_global(
            "main",
            &format!(
                "
                try {{
                    foo()
                }} catch (e) {{
                    if (e instanceof {js_type}) {{
                        result = true
                    }}
                }}"
            ),
        )?;
        assert!(ctx.global_object()?.get_property("result")?.as_bool()?);
        Ok(())
    }

    #[test]
    fn test_wrap_callback_handles_error_messages_with_null_bytes() -> anyhow::Result<()> {
        let ctx = JSContextRef::default();
        ctx.global_object()?.set_property(
            "foo",
            super::wrap(move |_, _, _| anyhow::bail!("Error containing \u{0000} with more"))
                .into_js_value(&ctx)?,
        )?;
        let res = ctx.eval_global("main", "foo();");
        let err = res.unwrap_err();
        assert_eq!(
            "Uncaught InternalError: Error containing  - truncated due to null byte\n    at <eval> (main)\n",
            err.to_string()
        );
        Ok(())
    }

    #[test]
    fn test_closure_args_and_return_value() -> anyhow::Result<()> {
        let ctx = JSContextRef::default();

        let expected = {
            let mut map = HashMap::new();
            map.insert("a".into(), JSValue::Float(123.45));
            map.insert("b".into(), JSValue::Null);
            map.insert("c".into(), JSValue::String("hello".into()));
            JSValue::Object(map)
        };
        let expected_clone = expected.clone();
        ctx.global_object()?.set_property(
            "foo",
            super::wrap(move |_, _, args| {
                assert_eq!(args[0].to_string(), "[object Object]");
                Ok(expected_clone.clone())
            })
            .into_js_value(&ctx)?,
        )?;

        ctx.eval_global(
            "main",
            r#"
            const value = foo({ foo: 'bar' });
            const valueRoundtrip = JSON.parse(JSON.stringify(value));
            result = valueRoundtrip;
            "#,
        )?;
        let result = ctx.global_object()?.get_property("result")?;
        let result = qjs_convert::from_qjs_value(&result)?;

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn text_two_closures() -> anyhow::Result<()> {
        let ctx = JSContextRef::default();

        let object = ctx.object_value()?;
        object.set_property(
            "a",
            super::wrap(move |_, _, _| Ok(JSValue::Int(2))).into_js_value(&ctx)?,
        )?;
        object.set_property(
            "b",
            super::wrap(move |_, _, _| Ok(JSValue::Int(10))).into_js_value(&ctx)?,
        )?;
        ctx.global_object()?.set_property("foo", object)?;

        ctx.eval_global(
            "main",
            r#"
            result = foo.a() + foo.b();
            "#,
        )?;
        let result = ctx
            .global_object()?
            .get_property("result")?
            .as_i32_unchecked();

        assert_eq!(result, 12);

        Ok(())
    }
}
