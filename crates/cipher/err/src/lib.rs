mod ext;

use std::fmt::{Debug, Display, Formatter};

pub enum Error<U: ToString = String> {
    UnexpectedAEAD(U),
}

pub type Result<T, U = String> = core::result::Result<T, Error<U>>;

impl<U: ToString> Display for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Error::UnexpectedAEAD(error) => error.to_string(),
        };
        write!(f, "{}", error)
    }
}

impl<U: ToString> Debug for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<U: ToString> std::error::Error for Error<U> {}

impl From<Error<&str>> for Error {
    fn from(value: Error<&str>) -> Self {
        match value {
            Error::UnexpectedAEAD(e) => Error::UnexpectedAEAD(e.to_string()),
        }
    }
}
