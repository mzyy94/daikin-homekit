use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    Tree {
        pn: String, // name
        #[serde(skip_serializing)]
        pt: u8, // type
        pch: Vec<Property>, // children
    },
    Item {
        pn: String, // name
        #[serde(skip_serializing)]
        pt: u8, // type
        pv: Option<PropValue>, // value
        #[serde(skip_serializing)]
        md: Option<Metadata>, // metadata
    },
}

impl Property {
    pub fn new(name: &str, value: PropValue) -> Property {
        Property::Item {
            pn: name.to_string(),
            pt: 2,
            pv: Some(value),
            md: None,
        }
    }

    pub fn new_tree(name: &str) -> Property {
        Property::Tree {
            pn: name.to_string(),
            pt: 3,
            pch: vec![],
        }
    }

    pub fn find(&self, name: &str) -> Option<&Property> {
        match self {
            Property::Tree { pch, .. } => pch.iter().find(|p| match p {
                Property::Tree { pn, .. } => pn == name,
                Property::Item { pn, .. } => pn == name,
            }),
            _ => None,
        }
    }

    pub fn step(&self) -> u8 {
        match self {
            Property::Item { md: Some(md), .. } => md.st,
            _ => 0,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Property::Item {
                pv: Some(PropValue::String(str)),
                ..
            } => str.len(),
            _ => 0,
        }
    }

    pub fn get_f64(&self) -> Option<f64> {
        match self {
            Property::Item {
                pv: Some(PropValue::String(pv)),
                md: Some(md),
                ..
            } => {
                if md.pt == "b" && !(md.mi == None && md.mx == None) {
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

    pub fn get_string(&self) -> Option<String> {
        match self {
            Property::Item {
                pv: Some(PropValue::String(pv)),
                md: Some(md),
                ..
            } => {
                if md.pt == "s" {
                    Some(String::from(pv))
                } else if md.pt == "b" && md.st == 0 && md.mi == None && md.mx == None {
                    todo!() // decode hex string
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl std::fmt::Debug for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Property::Tree { pn, pch, .. } => f
                .debug_struct("Tree")
                .field("name", pn)
                .field("pch", pch)
                .finish(),
            Property::Item {
                pn,
                pv: Some(PropValue::String(pv)),
                ..
            } => match self.get_f64() {
                Some(val) => f
                    .debug_struct("Item")
                    .field("name", pn)
                    .field("pv", &val)
                    .finish(),
                None => f
                    .debug_struct("Item")
                    .field("name", pn)
                    .field("pv", pv)
                    .finish(),
            },

            Property::Item {
                pn,
                pv: Some(PropValue::Integer(pv)),
                ..
            } => f
                .debug_struct("Item")
                .field("name", pn)
                .field("pv", pv)
                .finish(),
            Property::Item { pn, pv, .. } => f
                .debug_struct("Item")
                .field("name", pn)
                .field("pv", pv)
                .finish(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PropValue {
    String(String),
    Integer(i32),
}

impl PropValue {
    pub fn from(value: f64, step: u8, size: usize) -> PropValue {
        let mut wtr = vec![];

        let step_base = f64::from(step & 0xf);
        let exp: i32 = ((step & 0xf0) >> 4).into();
        let step_coefficient = if exp < 8 {
            10.0_f64.powi(exp)
        } else {
            10.0_f64.powi(exp - 16)
        };
        let step = step_base * step_coefficient;

        let value: i32 = if step == 0.0 {
            value as i32
        } else {
            (value / step) as i32
        };
        match size {
            2 => wtr.write_i8(value as i8).unwrap(),
            4 => wtr.write_i16::<LittleEndian>(value as i16).unwrap(),
            6 => wtr.write_i24::<LittleEndian>(value).unwrap(),
            8 => wtr.write_i32::<LittleEndian>(value).unwrap(),
            _ => {}
        };
        PropValue::String(hex::encode(wtr))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pt: String, // type
    #[serde(default)]
    st: u8, // step
    mi: Option<String>, // min
    mx: Option<String>, // max
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn get_f64() {
        let json = json!({
            "pn": "p_02",
            "pt": 2,
            "pv": "31",
            "md": {
                "pt": "b",
                "st": 245,
                "mi": "24",
                "mx": "40"
            }
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_f64(), Some(24.5));

        let json = json!({
            "pn": "root_entity_name",
            "pt": 3,
            "pv": "e_1002",
            "md": {
                "pt": "s"
            }
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_f64(), None);
    }

    #[test]
    fn get_string() {
        let json = json!({
            "pn": "root_entity_name",
            "pt": 3,
            "pv": "e_1002",
            "md": {
                "pt": "s"
            }
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_string(), Some("e_1002".into()));

        let json = json!({
            "pn": "data_model_code",
            "pt": 3,
            "pv": 26,
            "md": {
                "pt": "i"
            }
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_string(), None);
    }

    #[test]
    fn propvalue() {
        let json = json!({
            "pn": "p_01",
            "pt": 3,
            "pv": "2600",
            "md": {
                "pt": "b",
                "st": 245,
                "mi": "EEFF",
                "mx": "4E00"
            }
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");
        let pv = PropValue::from(p.get_f64().unwrap(), p.step(), p.size());
        let expect = PropValue::String("2600".into());

        assert_eq!(pv, expect);
    }

    #[test]
    fn debug_display() {
        let json = json!({
            "pn": "e_A00D",
            "pt": 1,
            "pch": [
                {
                    "pn": "p_01",
                    "pt": 3,
                    "pv": "2600",
                    "md": {
                        "pt": "b",
                        "st": 245,
                        "mi": "EEFF",
                        "mx": "4E00"
                    }
                }
            ]
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");

        assert_eq!(
            format!("{:?}", p),
            r#"Tree { name: "e_A00D", pch: [Item { name: "p_01", pv: 19.0 }] }"#
        );
    }
}
