use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub(super) struct StreamError<'a> {
    kind: ErrorKind,
    error: &'a str,
}

impl<'a> StreamError<'a> {
    #[allow(dead_code)]
    pub fn new(kind: ErrorKind, error: &'a str) -> Self {
        StreamError { kind, error }
    }
}

impl<'a> std::fmt::Display for StreamError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl<'a> From<StreamError<'a>> for Error {
    fn from(value: StreamError) -> Self {
        Error::new(value.kind, value.error)
    }
}

impl<'a> std::error::Error for StreamError<'a> {}

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
pub(super) const ERR_FT_TIMEOUT: StreamError = StreamError {
    kind: ErrorKind::TimedOut,
    error: "future request has timed out",
};
