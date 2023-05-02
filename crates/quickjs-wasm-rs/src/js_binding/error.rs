use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// `JSError` represents various types of JavaScript errors that can occur during the execution
/// of JavaScript code in QuickJS.
///
/// This enum provides a convenient way to classify and handle different types of JavaScript errors
/// that may be encountered. Each variant includes an associated error message to help with
/// debugging and error reporting.
#[derive(Debug)]
pub enum JSError {
    /// A syntax error that occurs when parsing or executing invalid JavaScript code.
    Syntax(String),
    /// A type error that occurs when an operation is performed on an incompatible type.
    Type(String),
    /// A reference error that occurs when trying to access an undefined variable or property.
    Reference(String),
    /// A range error that occurs when a value is outside the allowable range or when an invalid length is specified for an array or string.
    Range(String),
    /// An internal error that occurs due to an issue within the JavaScript engine or the Rust-QuickJS integration.
    Internal(String),
}

impl Display for JSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg)
            | Self::Range(msg)
            | Self::Reference(msg)
            | Self::Syntax(msg)
            | Self::Type(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for JSError {}
