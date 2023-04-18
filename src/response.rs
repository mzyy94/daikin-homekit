use crate::property::Property;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DaikinResponse {
    pub responses: Vec<Response>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Response {
    pub fr: String,           // from
    pub pc: Option<Property>, // content
    pub rsc: u32,             // response status code
}
