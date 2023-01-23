use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum JSError {
    Syntax(String),
    Type(String),
    Reference(String),
    Range(String),
    Internal(String),
}

impl Display for JSError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(msg)
            | Self::Range(msg)
            | Self::Reference(msg)
            | Self::Syntax(msg)
            | Self::Type(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for JSError {}
