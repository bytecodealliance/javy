use anyhow::Result;

pub trait JsValue {
    fn as_i32(&self) -> i32;
}

pub trait JsRuntime {
    fn default() -> Self;
    fn eval(&self, name: &str, contents: &str) -> Box<dyn JsValue>;
    fn global_object(&self) -> Result<Box<dyn JsValue>>;
    fn wrap_callback<F>(&self, f: F) -> Result<Box<dyn JsValue>>
    where
        F: Fn(&Self, &dyn JsValue, &Vec<dyn JsValue>) -> Result<Box<dyn JsValue>> + 'static;
}