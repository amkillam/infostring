use core::fmt;

#[cfg(not(feature = "std"))]
use crate::alloc::string::{String, ToString};

#[derive(Debug, PartialEq)]
pub enum Error {
    Message(String),
    ParseError(String),
    Eof,
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::ParseError(msg) => write!(f, "Failed to parse: {}", msg),
            Error::Eof => write!(f, "Unexpected end of string"),
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
