impl From<common::howler::Error> for super::Error {
    fn from(value: common::howler::Error) -> super::Error {
        super::Error::HowlerInternal(value)
    }
}

impl From<std::net::AddrParseError> for super::Error {
    fn from(value: std::net::AddrParseError) -> super::Error {
        super::Error::AddressNotParsed(value.to_string())
    }
}

impl From<std::io::Error> for super::Error {
    fn from(value: std::io::Error) -> super::Error {
        super::Error::AsyncIOFailed(value.to_string())
    }
}

impl From<std::string::FromUtf8Error> for super::Error {
    fn from(value: std::string::FromUtf8Error) -> super::Error {
        super::Error::StringNotUTF8(value.to_string())
    }
}
