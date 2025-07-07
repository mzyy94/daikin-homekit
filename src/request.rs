use crate::property::Property;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct DaikinRequest {
    pub requests: Vec<Request>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Request {
    pub op: u8,
    pub pc: Property,
    pub to: String,
}
