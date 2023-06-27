use async_std::io;
use std::string;

#[derive(Debug)]
pub enum ErrorKind {
    StreamFailed,
    AddressNotParsed,
    AsyncIOFailed,
    StringNotUTF8,
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

impl From<common::stream::Error> for Error {
    fn from(value: common::stream::Error) -> Self {
        let kind = ErrorKind::StreamFailed;
        Error::new(kind, value.to_string())
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        let kind = ErrorKind::AddressNotParsed;
        Error::new(kind, value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        let kind = ErrorKind::AsyncIOFailed;
        Error::new(kind, value.to_string())
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(value: string::FromUtf8Error) -> Self {
        let kind = ErrorKind::StringNotUTF8;
        Error::new(kind, value.to_string())
    }
}

impl std::error::Error for Error {}
