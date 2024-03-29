mod ext;

use std::fmt::{Debug, Display, Formatter};

pub enum Error<U: ToString = String> {
    CipherError(cipher_err::Error<U>),
    SocketError(socket_err::Error<U>),
    StreamError(stream_err::Error<U>),
}

pub type Result<T, U = String> = core::result::Result<T, Error<U>>;

impl<U: ToString> Display for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Error::CipherError(error) => error.to_string(),
            Error::SocketError(error) => error.to_string(),
            Error::StreamError(error) => error.to_string(),
        };
        write!(f, "{}", error)
    }
}

impl<U: ToString> Debug for Error<U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: implement debug
        write!(f, "{}", self)
    }
}

impl<U: ToString> std::error::Error for Error<U> {}
