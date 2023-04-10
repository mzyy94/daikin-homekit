use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DaikinResponse {
    responses: Vec<Response>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    fr: String,   // from
    pc: Property, // content
    rsc: u32,     // ??
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Property {
    Tree {
        pn: String,         // name
        pt: u8,             // type
        pch: Vec<Property>, // children
    },
    Item {
        pn: String,            // name
        pt: u8,                // type
        pv: Option<PropValue>, // value
        md: Option<Metadata>,  // metadata
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum PropValue {
    String(String),
    Integer(i32),
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    pt: String, // type
    #[serde(default)]
    st: u8, // step
    mi: Option<String>, // min
    mx: Option<String>, // max
}
