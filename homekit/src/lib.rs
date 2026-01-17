pub mod characteristic;
pub mod fan_mapping;
pub mod mode_mapping;
#[macro_use]
extern crate log;

// Re-export from daikin-client for convenience
pub use daikin_client::{Daikin, DaikinInfo, DaikinStatus, HttpClient, ReqwestClient, discovery};
