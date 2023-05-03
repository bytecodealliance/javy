use std::io::{self, Write};

use crate::APIConfig;

/// A selection of possible destination streams for `console.log` and
/// `console.error`.
#[derive(Debug)]
pub enum LogStream {
    /// The standard output stream.
    StdOut,
    /// The standard error stream.
    StdErr,
}

impl LogStream {
    pub(super) fn to_stream(&self) -> Box<dyn Write + 'static> {
        match self {
            Self::StdErr => Box::new(io::stderr()),
            Self::StdOut => Box::new(io::stdout()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ConsoleConfig {
    pub(super) log_stream: LogStream,
    pub(super) error_stream: LogStream,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            log_stream: LogStream::StdOut,
            error_stream: LogStream::StdErr,
        }
    }
}

impl APIConfig {
    /// Sets the destination stream for `console.log`.
    pub fn log_stream(&mut self, stream: LogStream) -> &mut Self {
        self.console.log_stream = stream;
        self
    }

    /// Sets the destination stream for `console.error`.
    pub fn error_stream(&mut self, stream: LogStream) -> &mut Self {
        self.console.error_stream = stream;
        self
    }
}
