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
            Self::Internal(msg) => write!(f, "{}", msg),
            Self::Range(msg) => write!(f, "{}", msg),
            Self::Reference(msg) => write!(f, "{}", msg),
            Self::Syntax(msg) => write!(f, "{}", msg),
            Self::Type(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for JSError {}
