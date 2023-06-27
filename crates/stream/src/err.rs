use async_std::channel::TrySendError;
use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE};

#[derive(Debug)]
pub enum ErrorKind {
    AlreadyInitialized,
    ConcurrencyModelChanged,
    Other,
    ChannelIsFull,
    ChannelIsClosed,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    error: String,
}

impl Error {
    pub fn new(kind: ErrorKind, error: String) -> Self {
        Error { kind, error }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<windows::core::Error> for Error {
    fn from(value: windows::core::Error) -> Self {
        let kind = match value.code() {
            S_FALSE => ErrorKind::AlreadyInitialized,
            RPC_E_CHANGED_MODE => ErrorKind::ConcurrencyModelChanged,
            _ => ErrorKind::Other,
        };
        Error::new(kind, value.message().to_string())
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::new(ErrorKind::Other, value.to_string())
    }
}

impl<T> From<TrySendError<T>> for Error {
    fn from(value: TrySendError<T>) -> Self {
        let kind = match value {
            TrySendError::Full(_) => ErrorKind::ChannelIsFull,
            TrySendError::Closed(_) => ErrorKind::ChannelIsClosed,
        };
        Error::new(kind, value.to_string())
    }
}

impl std::error::Error for Error {}
