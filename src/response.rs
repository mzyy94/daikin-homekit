use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Serialize, Deserialize, Debug)]
pub struct DaikinResponse {
    responses: Vec<Response>,
}
impl DaikinResponse {
    fn find_property(&self, name: &str) -> Option<&Property> {
        self.responses.iter().find(|&r| r.fr == name).map(|r| &r.pc)
    }

    pub fn get(&self, name: &str, path: &str) -> Option<&Property> {
        let root = self.find_property(name);
        root.and_then(|prop| prop.search(path))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    fr: String,   // from
    pc: Property, // content
    rsc: u32,     // ??
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Property {
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

impl Property {
    fn find(&self, name: &str) -> Option<&Property> {
        match self {
            Property::Tree { pch, .. } => pch.iter().find(|p| match p {
                Property::Tree { pn, .. } => pn == name,
                Property::Item { pn, .. } => pn == name,
            }),
            _ => None,
        }
    }

    fn search(&self, path: &str) -> Option<&Property> {
        path.split('/')
            .fold(Some(&self), |prop, name| prop.and_then(|p| p.find(name)))
    }

    pub fn get_f64(&self) -> Option<f64> {
        match self {
            Property::Item {
                pv: Some(PropValue::String(pv)),
                md: Some(md),
                ..
            } => {
                if md.pt == "b" {
                    let mut bytes = vec![0u8; pv.len() / 2];
                    hex::decode_to_slice(pv, &mut bytes as &mut [u8]).ok();
                    let mut rdr = Cursor::new(bytes);

                    let value: f64 = match pv.len() {
                        2 => f64::from(rdr.read_i8().unwrap()),
                        4 => f64::from(rdr.read_i16::<LittleEndian>().unwrap()),
                        6 => f64::from(rdr.read_i24::<LittleEndian>().unwrap()),
                        8 => f64::from(rdr.read_i32::<LittleEndian>().unwrap()),
                        _ => 0.0,
                    };

                    let step_base = f64::from(md.st & 0xf);
                    let exp: i32 = ((md.st & 0xf0) >> 4).into();
                    let step_coefficient = if exp < 8 {
                        10.0_f64.powi(exp)
                    } else {
                        10.0_f64.powi(exp - 16)
                    };
                    let step = step_base * step_coefficient;
                    if step == 0.0 {
                        Some(value)
                    } else {
                        Some(value * step)
                    }
                } else {
                    None
                }
            }
            Property::Item {
                pv: Some(PropValue::Integer(pv)),
                md: Some(md),
                ..
            } => {
                if md.pt == "i" {
                    Some(f64::from(*pv))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PropValue {
    String(String),
    Integer(i32),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pt: String, // type
    #[serde(default)]
    st: u8, // step
    mi: Option<String>, // min
    mx: Option<String>, // max
}
