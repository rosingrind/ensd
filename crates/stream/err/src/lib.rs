mod ext;

use std::fmt::{Debug, Display, Formatter};

pub enum Error<U: ToString = String> {
    AlreadyInitialized(U),
    ConcurrencyModelChanged(U),
    Other(U),
    ChannelIsFull(U),
    ChannelIsClosed(U),
}

pub type Result<T, U = String> = core::result::Result<T, Error<U>>;

impl<U: ToString> Display for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Error::AlreadyInitialized(error) => error.to_string(),
            Error::ConcurrencyModelChanged(error) => error.to_string(),
            Error::Other(error) => error.to_string(),
            Error::ChannelIsFull(error) => error.to_string(),
            Error::ChannelIsClosed(error) => error.to_string(),
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
            Error::AlreadyInitialized(e) => Error::AlreadyInitialized(e.to_string()),
            Error::ConcurrencyModelChanged(e) => Error::ConcurrencyModelChanged(e.to_string()),
            Error::Other(e) => Error::Other(e.to_string()),
            Error::ChannelIsFull(e) => Error::ChannelIsFull(e.to_string()),
            Error::ChannelIsClosed(error) => Error::ChannelIsClosed(error.to_string()),
        }
    }
}
