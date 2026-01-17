//! DSIOT - Daikin Smart IoT Protocol Library
//!
//! This crate provides protocol-agnostic abstractions for HVAC control,
//! with specific implementations for Daikin devices.

pub mod constraints;
pub mod mapping;
pub mod protocol;
pub mod state;
pub mod temperature;
pub mod types;

// Re-export commonly used types at crate root for convenience
pub use constraints::ValueConstraints;
pub use state::{DeviceState, PowerState, StateTransition, StateTransitionError};
pub use temperature::{TemperatureError, TemperatureTarget};
pub use types::{AutoModeWindSpeed, HorizontalDirection, Mode, VerticalDirection, WindSpeed};

// Re-export protocol types for backward compatibility
pub use protocol::{
    Binary, BinaryStep, DaikinInfo, DaikinRequest, DaikinResponse, DaikinStatus, Item, Metadata,
    PropValue, Property, SensorReadings, TemperatureSettings, WindSettings,
};

// Legacy module aliases for backward compatibility
#[doc(hidden)]
pub mod property {
    pub use crate::protocol::property::*;
}
#[doc(hidden)]
pub mod status {
    pub use crate::protocol::status::*;
    pub use crate::types::*;
}
#[doc(hidden)]
pub mod info {
    pub use crate::protocol::info::*;
}
#[doc(hidden)]
pub mod request {
    pub use crate::protocol::request::*;
}
#[doc(hidden)]
pub mod response {
    pub use crate::protocol::response::*;
}
