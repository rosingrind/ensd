use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub(super) struct StreamError {
    kind: ErrorKind,
    error: &'static str,
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Into<Error> for StreamError {
    fn into(self) -> Error {
        Error::new(self.kind, self.error)
    }
}

impl std::error::Error for StreamError {}

pub(super) const ERR_CONNECTION: StreamError = StreamError {
    kind: ErrorKind::TimedOut,
    error: "can't reach remote host in required number of attempts",
};
pub(super) const ERR_VALIDATION: StreamError = StreamError {
    kind: ErrorKind::InvalidInput,
    error: "remote host returned non-stage or invalid message",
};
pub(super) const ERR_PIPE_BROKE: StreamError = StreamError {
    kind: ErrorKind::BrokenPipe,
    error: "incorrect message exchange procedure ordering",
};
pub(super) const ERR_STUN_QUERY: StreamError = StreamError {
    kind: ErrorKind::Other,
    error: "can't decode any valid address from STUN message",
};
