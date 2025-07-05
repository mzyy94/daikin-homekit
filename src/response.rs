use crate::property::Property;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DaikinResponse {
    pub responses: Vec<Response>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Response {
    #[serde(rename = "fr")]
    pub from: String,
    #[serde(rename = "pc")]
    pub content: Option<Property>,
    #[serde(rename = "rsc")]
    pub status_code: u32, // response status code
}
