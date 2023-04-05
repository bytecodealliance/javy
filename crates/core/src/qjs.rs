use core::slice::SlicePattern;

use quickjs_wasm_rs::{Context, Value};
use anyhow::Result;

use super::runtime_trait::{JsRuntime, JsValue};

pub struct QJSRuntime {
    context: Context,
}

pub struct QJSValue {
    inner: Value
}

impl JsValue for QJSValue {
    fn as_i32(&self) -> i32 {
        self.inner.as_i32_unchecked()
    }
}

impl JsRuntime for QJSRuntime {
    fn default() -> Self {
        QJSRuntime {
            context: Context::default()
        }
    }

    fn eval(&self, name: &str, contents: &str) -> Box<dyn JsValue> {
        let val = self.context.eval_global(name, contents);
        Box::new(QJSValue { inner: val.unwrap() })
    }

    fn global_object(&self) -> Result<Box<dyn JsValue>> {
        let val = self.context.global_object()?;
        Ok(Box::new(QJSValue { inner: val }))
    }

    fn wrap_callback<F>(&self, f: F) -> Result<Box<dyn JsValue>>
        where
            F: Fn(&Self, &dyn JsValue, &Vec<dyn JsValue>) -> Result<Box<dyn JsValue>> + 'static {
        let trampoline = |ctx: &Context, this: &Value, args: &[Value]| {
            let this = QJSValue { inner: this.clone() };
            let args = args.iter().map(|arg| QJSValue { inner: arg.clone() }).collect::<Vec<_>>();
            f(self, &this, &args)
        };
        Ok(Box::new(QJSValue { inner: val }))
    }
}