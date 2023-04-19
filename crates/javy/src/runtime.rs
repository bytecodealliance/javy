use anyhow::Result;
use quickjs_wasm_rs::JSContextRef;

#[cfg(feature = "console")]
use crate::console::Console;
#[cfg(feature = "stream_io")]
use crate::stream_io::StreamIO;
#[cfg(feature = "text_encoding")]
use crate::text_encoding::TextEncoding;
use crate::{Config, JSApiSet};

#[derive(Debug)]
pub struct Runtime {
    context: JSContextRef,
}

impl Runtime {
    pub fn new(#[allow(unused_variables)] config: &Config) -> Result<Self> {
        let context = JSContextRef::default();

        #[cfg(feature = "console")]
        Console::new().register(&context, &config)?;

        #[cfg(feature = "stream_io")]
        StreamIO::new().register(&context, &config)?;

        #[cfg(feature = "text_encoding")]
        TextEncoding::new().register(&context, &config)?;

        Ok(Self { context })
    }

    pub fn context(&self) -> &JSContextRef {
        &self.context
    }
}
