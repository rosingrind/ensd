use crate::{Error, ErrorKind};
use async_std::io;
use std::string;

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

impl From<bytecodec::Error> for Error {
    fn from(value: bytecodec::Error) -> Self {
        let kind = match value.kind() {
            bytecodec::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
            bytecodec::ErrorKind::InconsistentState => ErrorKind::InconsistentState,
            bytecodec::ErrorKind::UnexpectedEos => ErrorKind::UnexpectedEos,
            bytecodec::ErrorKind::EncoderFull => ErrorKind::EncoderFull,
            bytecodec::ErrorKind::DecoderTerminated => ErrorKind::DecoderTerminated,
            bytecodec::ErrorKind::IncompleteDecoding => ErrorKind::IncompleteDecoding,
            bytecodec::ErrorKind::Other => ErrorKind::Other,
        };
        Error::new(kind, value.to_string())
    }
}

impl From<stun_codec::BrokenMessage> for Error {
    fn from(value: stun_codec::BrokenMessage) -> Self {
        let kind = ErrorKind::BrokenMessage;
        Error::new(kind, format!("{:?}", value))
    }
}
