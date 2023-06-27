use crate::{Error, ErrorKind};

impl From<aead::Error> for Error<String> {
    fn from(value: aead::Error) -> Self {
        let kind = ErrorKind::UnexpectedAEAD;
        Error::new(kind, value.to_string())
    }
}
