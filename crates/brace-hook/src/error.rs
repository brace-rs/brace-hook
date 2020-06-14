use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum Error {
    Message(String),
}

impl Error {
    pub fn message<T>(message: T) -> Self
    where
        T: Into<String>,
    {
        Self::Message(message.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Message(message) => write!(f, "{}", message),
        }
    }
}

impl StdError for Error {}
