use std::{io, net::AddrParseError, str::Utf8Error};

use crate::response::Response;
use thiserror::Error;
use tokio::time::error::Elapsed;

#[derive(Error, Debug)]
/// The Error type
pub enum Error {
    /// Occurred by UDP socket access
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    /// Occurred by std::str::from_utf8
    #[error("utf8 decoding error: {0}")]
    Utf8(#[from] Utf8Error),
    /// HTTP request error
    #[error("reqest error: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Bad HTTP Status Code
    #[error("http responded with non-200 status code: {0}")]
    HTTPError(reqwest::StatusCode),
    /// Serialize, Deserialize failure
    #[error("json error: {0}")]
    JSONError(#[from] serde_json::Error),
    /// Bad Response Status Code (rsc)
    #[error("api responded with non-200x rsc: {0:?}")]
    RSCError(Vec<Response>),
    /// IPv4 Address parse failure
    #[error("parse ip address error: {0}")]
    AddrParseError(#[from] AddrParseError),
    /// Missing Property
    #[error("no property found error")]
    NoProperty,
    /// HAP Server error
    #[error("hap server error: {0}")]
    HAPError(#[from] hap::Error),
    /// Device discovery timeout error
    #[error("discovery timeout error")]
    DiscoveryTimeout(#[from] Elapsed),
    /// Unknown error
    #[error("unknown error")]
    Unknown,
}
