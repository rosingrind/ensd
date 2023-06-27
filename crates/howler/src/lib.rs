mod cipher;
mod common;
mod socket;
mod stream;

use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum ErrorKind {
    AlreadyInitialized,
    ConcurrencyModelChanged,
    Other,
    ChannelIsFull,
    ChannelIsClosed,
    TimedOut,
    InvalidInput,
    BrokenPipe,
    StreamFailed,
    AddressNotParsed,
    AsyncIOFailed,
    StringNotUTF8,
    UnexpectedAEAD,
    InconsistentState,
    UnexpectedEos,
    EncoderFull,
    DecoderTerminated,
    IncompleteDecoding,
    BrokenMessage,
}

#[allow(dead_code)]
pub struct Error<U: ToString = String> {
    kind: ErrorKind,
    error: U,
}

pub type Result<T, U = String> = core::result::Result<T, Error<U>>;

impl<U: ToString> Error<U> {
    pub fn new(kind: ErrorKind, error: U) -> Self {
        Error { kind, error }
    }
}

impl<U: ToString> Display for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error.to_string())
    }
}

impl<U: ToString> Debug for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error {{ kind: {:?}, error: {} }}",
            self.kind,
            self.error.to_string()
        )
    }
}

impl<U: ToString> std::error::Error for Error<U> {}

impl From<Error<&str>> for Error {
    fn from(value: Error<&str>) -> Self {
        Error::new(value.kind, value.error.to_string())
    }
}

pub mod consts {
    pub use crate::socket::{ERR_CONNECTION, ERR_PIPE_BROKE, ERR_STUN_QUERY, ERR_VALIDATION};
}
