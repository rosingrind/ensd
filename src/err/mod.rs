mod ext;

use std::fmt::{Debug, Display, Formatter};

pub enum Error<U: ToString = String> {
    HowlerInternal(common::howler::Error<U>),
    AddressNotParsed(U),
    AsyncIOFailed(U),
    StringNotUTF8(U),
}

pub type Result<T, U = String> = core::result::Result<T, Error<U>>;

impl<U: ToString> Display for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Error::HowlerInternal(error) => error.to_string(),
            Error::AddressNotParsed(error) => error.to_string(),
            Error::AsyncIOFailed(error) => error.to_string(),
            Error::StringNotUTF8(error) => error.to_string(),
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
