use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSError {
    #[error("SyntaxError: {0}")]
    Syntax(String),
    #[error("TypeError: {0}")]
    Type(String),
    #[error("ReferenceError: {0}")]
    Reference(String),
    #[error("RangeError: {0}")]
    Range(String),
    #[error("InternalError: {0}")]
    Internal(String),
}

impl JSError {
    pub(super) fn msg(&self) -> &str {
        match self {
            Self::Syntax(err) => err,
            Self::Type(err) => err,
            Self::Reference(err) => err,
            Self::Range(err) => err,
            Self::Internal(err) => err,
        }
    }
}
