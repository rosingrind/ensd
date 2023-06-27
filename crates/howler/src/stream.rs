use crate::{Error, ErrorKind};
use async_std::channel::TrySendError;
use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE};

impl From<windows::core::Error> for Error<String> {
    fn from(value: windows::core::Error) -> Self {
        let kind = match value.code() {
            S_FALSE => ErrorKind::AlreadyInitialized,
            RPC_E_CHANGED_MODE => ErrorKind::ConcurrencyModelChanged,
            _ => ErrorKind::Other,
        };
        Error::new(kind, value.message().to_string())
    }
}

impl From<Box<dyn std::error::Error>> for Error<String> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::new(ErrorKind::Other, value.to_string())
    }
}

impl<T> From<TrySendError<T>> for Error<String> {
    fn from(value: TrySendError<T>) -> Self {
        let kind = match value {
            TrySendError::Full(_) => ErrorKind::ChannelIsFull,
            TrySendError::Closed(_) => ErrorKind::ChannelIsClosed,
        };
        Error::new(kind, value.to_string())
    }
}
