#[cfg(target_os = "macos")]
mod osx;
#[cfg(target_os = "windows")]
mod win;

use async_std::channel::{RecvError, SendError, TryRecvError, TrySendError};

use super::Error;

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
