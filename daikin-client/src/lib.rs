//! Daikin AC network client library.
//!
//! This crate provides HTTP client and device discovery for Daikin air conditioners.

#[macro_use]
extern crate log;

mod client;
mod discovery;

pub use client::{Daikin, HttpClient, ReqwestClient};
pub use discovery::discovery;

// Re-export commonly used types from dsiot
pub use dsiot::{DaikinInfo, DaikinStatus};
