use anyhow::Result;
pub use api_config::APIConfig;
use javy::{quickjs::JSContextRef, Runtime};
pub use runtime_ext::RuntimeExt;

#[cfg(feature = "console")]
use console::Console;
#[cfg(feature = "stream_io")]
use stream_io::StreamIO;
#[cfg(feature = "text_encoding")]
use text_encoding::TextEncoding;

#[cfg(feature = "console")]
pub use console::LogStream;

mod api_config;
#[cfg(feature = "console")]
mod console;
mod runtime_ext;
#[cfg(feature = "stream_io")]
mod stream_io;
#[cfg(feature = "text_encoding")]
mod text_encoding;

pub(crate) trait JSApiSet {
    fn register(&self, context: &JSContextRef, config: &APIConfig) -> Result<()>;
}

pub fn add_to_runtime(runtime: &Runtime, config: &APIConfig) -> Result<()> {
    let context = runtime.context();

    #[cfg(feature = "console")]
    Console::new().register(&context, &config)?;

    #[cfg(feature = "stream_io")]
    StreamIO::new().register(&context, &config)?;

    #[cfg(feature = "text_encoding")]
    TextEncoding::new().register(&context, &config)?;

    Ok(())
}
