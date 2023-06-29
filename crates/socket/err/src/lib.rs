mod ext;

use std::fmt::{Debug, Display, Formatter};

pub enum Error<U: ToString = String> {
    TimedOut(U),
    InvalidInput(U),
    BrokenPipe(U),
    Other(U),
    InconsistentState(U),
    UnexpectedEos(U),
    EncoderFull(U),
    DecoderTerminated(U),
    IncompleteDecoding(U),
    BrokenMessage(U),
}

pub type Result<T, U = String> = core::result::Result<T, Error<U>>;

impl<U: ToString> Display for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Error::TimedOut(error) => error.to_string(),
            Error::InvalidInput(error) => error.to_string(),
            Error::BrokenPipe(error) => error.to_string(),
            Error::Other(error) => error.to_string(),
            Error::InconsistentState(error) => error.to_string(),
            Error::UnexpectedEos(error) => error.to_string(),
            Error::EncoderFull(error) => error.to_string(),
            Error::DecoderTerminated(error) => error.to_string(),
            Error::IncompleteDecoding(error) => error.to_string(),
            Error::BrokenMessage(error) => error.to_string(),
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
            Error::TimedOut(e) => Error::TimedOut(e.to_string()),
            Error::InvalidInput(e) => Error::InvalidInput(e.to_string()),
            Error::BrokenPipe(e) => Error::BrokenPipe(e.to_string()),
            Error::Other(e) => Error::Other(e.to_string()),
            Error::InconsistentState(error) => Error::InconsistentState(error.to_string()),
            Error::UnexpectedEos(error) => Error::UnexpectedEos(error.to_string()),
            Error::EncoderFull(error) => Error::EncoderFull(error.to_string()),
            Error::DecoderTerminated(error) => Error::DecoderTerminated(error.to_string()),
            Error::IncompleteDecoding(error) => Error::IncompleteDecoding(error.to_string()),
            Error::BrokenMessage(error) => Error::BrokenMessage(error.to_string()),
        }
    }
}

pub mod consts {
    pub use crate::ext::{ERR_CONNECTION, ERR_PIPE_BROKE, ERR_STUN_QUERY, ERR_VALIDATION};
}
