use async_std::future;
use std::io;

use super::Error;

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Error {
        match value.kind() {
            io::ErrorKind::TimedOut => Error::TimedOut(value.to_string()),
            io::ErrorKind::InvalidInput => Error::InvalidInput(value.to_string()),
            io::ErrorKind::BrokenPipe => Error::BrokenPipe(value.to_string()),
            _ => Error::Other(value.to_string()),
        }
    }
}

impl From<future::TimeoutError> for Error<String> {
    fn from(value: future::TimeoutError) -> Self {
        Error::TimedOut(value.to_string())
    }
}

impl From<bytecodec::Error> for Error {
    fn from(value: bytecodec::Error) -> Self {
        match value.kind() {
            bytecodec::ErrorKind::InvalidInput => Error::InvalidInput(value.to_string()),
            bytecodec::ErrorKind::InconsistentState => Error::InconsistentState(value.to_string()),
            bytecodec::ErrorKind::UnexpectedEos => Error::UnexpectedEos(value.to_string()),
            bytecodec::ErrorKind::EncoderFull => Error::EncoderFull(value.to_string()),
            bytecodec::ErrorKind::DecoderTerminated => Error::DecoderTerminated(value.to_string()),
            bytecodec::ErrorKind::IncompleteDecoding => {
                Error::IncompleteDecoding(value.to_string())
            }
            bytecodec::ErrorKind::Other => Error::Other(value.to_string()),
        }
    }
}

impl From<stun_codec::BrokenMessage> for Error {
    fn from(value: stun_codec::BrokenMessage) -> Self {
        Error::BrokenMessage(format!("{:?}", value))
    }
}

pub const ERR_CONNECTION: Error<&str> =
    Error::TimedOut("can't reach remote host in required number of attempts");
pub const ERR_VALIDATION: Error<&str> =
    Error::InvalidInput("remote host returned non-stage or invalid message");
pub const ERR_PIPE_BROKE: Error<&str> =
    Error::BrokenPipe("incorrect message exchange procedure ordering");
pub const ERR_STUN_QUERY: Error<&str> =
    Error::Other("can't decode any valid address from STUN message");
