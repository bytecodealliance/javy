mod js_binding;
mod serialize;

pub use crate::js_binding::context::Context;
pub use crate::js_binding::exception::Exception;
pub use crate::js_binding::value::Value;
pub use crate::serialize::de::Deserializer;
pub use crate::serialize::ser::Serializer;

#[deprecated = "This is a hack to enable Javy Core to compile and will be removed when it's no longer necessary"]
pub mod sys;

#[cfg(feature = "messagepack")]
pub mod messagepack;

#[cfg(feature = "json")]
pub mod json;
