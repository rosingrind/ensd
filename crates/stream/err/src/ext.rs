use async_std::channel::{RecvError, SendError, TryRecvError, TrySendError};
use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE};

use super::Error;

impl From<windows::core::Error> for Error<String> {
    fn from(value: windows::core::Error) -> Self {
        match value.code() {
            S_FALSE => Error::AlreadyInitialized(value.message().to_string()),
            RPC_E_CHANGED_MODE => Error::ConcurrencyModelChanged(value.message().to_string()),
            _ => Error::Other(value.message().to_string()),
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error<String> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::Other(value.to_string())
    }
}

impl<T> From<TrySendError<T>> for Error<String> {
    fn from(value: TrySendError<T>) -> Self {
        match value {
            TrySendError::Full(_) => Error::ChannelIsFull(value.to_string()),
            TrySendError::Closed(_) => Error::ChannelIsClosed(value.to_string()),
        }
    }
}

impl From<TryRecvError> for Error<String> {
    fn from(value: TryRecvError) -> Self {
        match value {
            TryRecvError::Empty => Error::ChannelIsEmpty(value.to_string()),
            TryRecvError::Closed => Error::ChannelIsClosed(value.to_string()),
        }
    }
}

impl<T> From<SendError<T>> for Error<String> {
    fn from(value: SendError<T>) -> Self {
        Error::ChannelIsClosed(value.to_string())
    }
}

impl From<RecvError> for Error<String> {
    fn from(value: RecvError) -> Self {
        Error::ChannelIsClosed(value.to_string())
    }
}