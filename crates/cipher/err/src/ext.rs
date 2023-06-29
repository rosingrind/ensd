use super::Error;

impl From<aead::Error> for Error<String> {
    fn from(value: aead::Error) -> Self {
        Error::UnexpectedAEAD(value.to_string())
    }
}
