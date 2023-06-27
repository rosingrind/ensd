use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub(super) struct NetError<'a> {
    kind: ErrorKind,
    error: &'a str,
}

impl<'a> NetError<'a> {
    #[allow(dead_code)]
    pub fn new(kind: ErrorKind, error: &'a str) -> Self {
        NetError { kind, error }
    }
}

impl<'a> std::fmt::Display for NetError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl<'a> From<NetError<'a>> for Error {
    fn from(value: NetError) -> Self {
        Error::new(value.kind, value.error)
    }
}

impl<'a> std::error::Error for NetError<'a> {}

pub(super) const ERR_CONNECTION: NetError = NetError {
    kind: ErrorKind::TimedOut,
    error: "can't reach remote host in required number of attempts",
};
pub(super) const ERR_VALIDATION: NetError = NetError {
    kind: ErrorKind::InvalidInput,
    error: "remote host returned non-stage or invalid message",
};
pub(super) const ERR_PIPE_BROKE: NetError = NetError {
    kind: ErrorKind::BrokenPipe,
    error: "incorrect message exchange procedure ordering",
};
pub(super) const ERR_STUN_QUERY: NetError = NetError {
    kind: ErrorKind::Other,
    error: "can't decode any valid address from STUN message",
};
pub(super) const ERR_FT_TIMEOUT: NetError = NetError {
    kind: ErrorKind::TimedOut,
    error: "future request has timed out",
};
