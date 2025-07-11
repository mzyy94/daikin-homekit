use crate::property::Property;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaikinRequest {
    pub requests: Vec<Request>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub op: u8,
    pub pc: Property,
    pub to: String,
}
