use std::{fmt, io, net::AddrParseError, str::Utf8Error};

#[derive(Debug)]
/// The Error type
pub enum Error {
    /// Occurred by UDP socket access
    IO(io::Error),
    /// Occurred by std::str::from_utf8
    Utf8(Utf8Error),
    /// HTTP request error
    RequestError(reqwest::Error),
    /// Bad HTTP Status Code
    HTTPError(reqwest::StatusCode),
    /// Serialize, Deserialize failure
    JSONError(serde_json::Error),
    /// IPv4 Address parse failure
    AddrParseError(AddrParseError),
    /// Missing Property
    NoProperty,
    /// HAP Server error
    HAPError(hap::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "io error: {}", err),
            Error::Utf8(err) => write!(f, "utf8 decoding error: {}", err),
            Error::RequestError(err) => write!(f, "reqest error: {}", err),
            Error::HTTPError(err) => write!(f, "http responded with non-200 status code: {}", err),
            Error::JSONError(err) => write!(f, "json error: {}", err),
            Error::AddrParseError(err) => write!(f, "parse ip address error: {}", err),
            Error::NoProperty => write!(f, "no property found error"),
            Error::HAPError(err) => write!(f, "hap server error: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JSONError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::RequestError(err)
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Error::AddrParseError(err)
    }
}

impl From<hap::Error> for Error {
    fn from(err: hap::Error) -> Self {
        Error::HAPError(err)
    }
}
