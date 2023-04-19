#[cfg(feature = "console")]
use crate::LogStream;

pub struct APIConfig {
    #[cfg(feature = "console")]
    pub(crate) log_stream: LogStream,
    #[cfg(feature = "console")]
    pub(crate) error_stream: LogStream,
}

impl APIConfig {
    #[cfg(feature = "console")]
    pub fn log_stream(&mut self, stream: LogStream) -> &mut Self {
        self.log_stream = stream;
        self
    }

    #[cfg(feature = "console")]
    pub fn error_stream(&mut self, stream: LogStream) -> &mut Self {
        self.error_stream = stream;
        self
    }
}

impl Default for APIConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "console")]
            log_stream: LogStream::StdOut,
            #[cfg(feature = "console")]
            error_stream: LogStream::StdErr,
        }
    }
}
