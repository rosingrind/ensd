use crate::{Error, ErrorKind};
use async_std::{future, io};

impl Into<io::ErrorKind> for ErrorKind {
    fn into(self) -> io::ErrorKind {
        match self {
            ErrorKind::TimedOut => io::ErrorKind::TimedOut,
            ErrorKind::InvalidInput => io::ErrorKind::InvalidInput,
            ErrorKind::BrokenPipe => io::ErrorKind::BrokenPipe,
            ErrorKind::Other => io::ErrorKind::Other,
            _ => todo!(),
        }
    }
}

impl<U: ToString> From<Error<U>> for io::Error {
    fn from(value: Error<U>) -> Self {
        io::Error::new(value.kind.into(), value.error.to_string())
    }
}

impl From<future::TimeoutError> for Error<String> {
    fn from(value: future::TimeoutError) -> Self {
        let kind = ErrorKind::TimedOut;
        Error::new(kind, value.to_string())
    }
}

pub const ERR_CONNECTION: Error<&str> = Error {
    kind: ErrorKind::TimedOut,
    error: "can't reach remote host in required number of attempts",
};
pub const ERR_VALIDATION: Error<&str> = Error {
    kind: ErrorKind::InvalidInput,
    error: "remote host returned non-stage or invalid message",
};
pub const ERR_PIPE_BROKE: Error<&str> = Error {
    kind: ErrorKind::BrokenPipe,
    error: "incorrect message exchange procedure ordering",
};
pub const ERR_STUN_QUERY: Error<&str> = Error {
    kind: ErrorKind::Other,
    error: "can't decode any valid address from STUN message",
};
