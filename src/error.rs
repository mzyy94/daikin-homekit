use crate::response::Response;
use thiserror::Error;

#[derive(Error, Debug)]
/// The Error type
pub enum Error {
    /// Bad HTTP Status Code
    #[error("http responded with non-200 status code: {0}")]
    HTTPError(reqwest::StatusCode),
    /// Bad Response Status Code (rsc)
    #[error("api responded with non-200x rsc: {0:?}")]
    RSCError(Vec<Response>),
    /// Missing Property
    #[error("no property found error")]
    NoProperty,
    /// Unknown error
    #[error("unknown error")]
    Unknown,
}
