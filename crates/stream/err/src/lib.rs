mod ext;

use std::fmt::{Debug, Display, Formatter};

pub enum Error<U: ToString = String> {
    AlreadyInitialized(U),
    ConcurrencyModelChanged(U),
    Other(U),
    ChannelIsEmpty(U),
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
            Error::ChannelIsEmpty(error) => error.to_string(),
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
