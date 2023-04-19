use anyhow::Result;
pub use config::Config;
#[cfg(feature = "console")]
pub use console::LogStream;
use quickjs_wasm_rs::JSContextRef;
pub use runtime::Runtime;

mod config;
#[cfg(feature = "console")]
mod console;
mod runtime;
#[cfg(feature = "stream_io")]
mod stream_io;
#[cfg(feature = "text_encoding")]
mod text_encoding;

pub(crate) trait JSApiSet {
    fn register(&self, context: &JSContextRef, config: &Config) -> Result<()>;
}
