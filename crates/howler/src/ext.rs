impl From<cipher_err::Error> for crate::Error {
    fn from(value: cipher_err::Error) -> crate::Error {
        crate::Error::CipherError(value)
    }
}

impl From<socket_err::Error> for crate::Error {
    fn from(value: socket_err::Error) -> crate::Error {
        crate::Error::SocketError(value)
    }
}

impl From<stream_err::Error> for crate::Error {
    fn from(value: stream_err::Error) -> crate::Error {
        crate::Error::StreamError(value)
    }
}
