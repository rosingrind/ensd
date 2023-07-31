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
