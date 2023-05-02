use std::io::{self, Write};

#[cfg(feature = "console")]
/// A selection of possible destination streams for `console.log` and
/// `console.error`.
#[derive(Debug)]
pub enum LogStream {
    /// The standard output stream.
    StdOut,
    /// The standard error stream.
    StdErr,
}

#[cfg(feature = "console")]
impl LogStream {
    pub(crate) fn to_stream(&self) -> Box<dyn Write + 'static> {
        match self {
            Self::StdErr => Box::new(io::stderr()),
            Self::StdOut => Box::new(io::stdout()),
        }
    }
}

/// A configuration for APIs added in this crate.
///
/// Example usage:
/// ```
/// # use javy_apis::{APIConfig, LogStream};
/// let api_config = APIConfig::default();
/// ```
#[derive(Debug)]
pub struct APIConfig {
    #[cfg(feature = "console")]
    pub(crate) log_stream: LogStream,
    #[cfg(feature = "console")]
    pub(crate) error_stream: LogStream,
}

impl APIConfig {
    #[cfg(feature = "console")]
    /// Sets the destination stream for `console.log`.
    pub fn log_stream(&mut self, stream: LogStream) -> &mut Self {
        self.log_stream = stream;
        self
    }

    #[cfg(feature = "console")]
    /// Sets the destination stream for `console.error`.
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
