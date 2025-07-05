use thiserror::Error;

#[derive(Error, Debug)]
/// The Error type
pub enum Error {
    /// Missing Property
    #[error("no property found error")]
    NoProperty,
    /// Unknown error
    #[error("unknown error")]
    Unknown,
}
