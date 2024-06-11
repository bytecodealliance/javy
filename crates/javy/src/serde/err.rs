use std::{error, fmt};
pub type Result<T> = std::result::Result<T, Error>;
use crate::quickjs::Error as JSError;

#[derive(Debug)]
pub enum Error {
    Custom(anyhow::Error),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(e) => {
                if let Some(e) = e.downcast_ref::<JSError>() {
                    formatter.write_str(&format!("JSError: {}", &e.to_string()))
                } else {
                    formatter.write_str(&e.to_string())
                }
            }
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Custom(e)
    }
}

impl From<JSError> for Error {
    fn from(value: JSError) -> Self {
        Error::Custom(anyhow::Error::new(value))
    }
}
