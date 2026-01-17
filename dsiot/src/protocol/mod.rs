//! Daikin-specific protocol implementation.

#[macro_use]
mod macros;
pub mod client;
pub mod info;
pub mod property;
pub mod request;
pub mod response;
pub mod status;

pub use client::{Daikin, HttpClient};
pub use info::DaikinInfo;
pub use property::{Binary, BinaryStep, Item, Metadata, PropValue, Property};
pub use request::DaikinRequest;
pub use response::DaikinResponse;
pub use status::{DaikinStatus, SensorReadings, TemperatureSettings, WindSettings};
