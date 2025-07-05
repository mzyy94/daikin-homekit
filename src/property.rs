use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Property {
    Tree {
        #[serde(rename = "pn")]
        name: String,
        #[serde(skip_serializing, rename = "pt")]
        type_: u8,
        #[serde(rename = "pch")]
        children: Vec<Property>,
    },
    Item {
        #[serde(rename = "pn")]
        name: String,
        #[serde(skip_serializing, rename = "pt")]
        type_: u8,
        #[serde(rename = "pv")]
        value: PropValue,
        #[serde(skip_serializing, rename = "md")]
        metadata: Metadata,
    },
}

fn hex2int(hex: &String) -> i32 {
    let size = hex.len();
    let mut bytes = vec![0u8; size / 2];
    hex::decode_to_slice(hex, &mut bytes as &mut [u8]).ok();
    let mut rdr = Cursor::new(bytes);

    match size {
        2 => rdr.read_i8().unwrap() as i32,
        4 => rdr.read_i16::<LittleEndian>().unwrap() as i32,
        6 => rdr.read_i24::<LittleEndian>().unwrap(),
        8 => rdr.read_i32::<LittleEndian>().unwrap(),
        _ => 0,
    }
}

impl Property {
    pub fn new(name: &str, value: PropValue) -> Property {
        Property::Item {
            name: name.to_string(),
            type_: 2,
            value: value,
            metadata: Metadata::default(),
        }
    }

    pub fn new_tree(name: &str) -> Property {
        Property::Tree {
            name: name.to_string(),
            type_: 3,
            children: vec![],
        }
    }

    pub fn find(&self, name: &str) -> Option<&Property> {
        match self {
            Property::Tree { children, .. } => children.iter().find(|p| match p {
                Property::Tree { name: n, .. } => name == n,
                Property::Item { name: n, .. } => name == n,
            }),
            _ => None,
        }
    }

    pub fn step(&self) -> f32 {
        match self {
            Property::Item { metadata: md, .. } => md.step(),
            _ => 0.0,
        }
    }

    pub fn meta(&self) -> (f32, Option<f32>, Option<f32>) {
        match self {
            Property::Item { metadata: md, .. } => md.get_tuple(),
            _ => (0.0, None, None),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Property::Item {
                value: PropValue::String(str),
                ..
            } => str.len(),
            _ => 0,
        }
    }

    pub fn get_f32(&self) -> Option<f32> {
        match self {
            Property::Item {
                value: PropValue::String(pv),
                metadata: md,
                ..
            } => {
                if md.type_ == "b" && !(md.min.is_none() && md.max.is_none()) {
                    let value = hex2int(pv) as f32;
                    let step = self.step();
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
                value: PropValue::Integer(pv),
                metadata: md,
                ..
            } => {
                if md.type_ == "i" {
                    Some(*pv as f32)
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
                value: PropValue::String(pv),
                metadata: md,
                ..
            } => {
                if md.type_ == "s" {
                    Some(String::from(pv))
                } else if md.type_ == "b" && md.step == 0 && md.min.is_none() && md.max.is_none() {
                    todo!() // decode hex string
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum PropValue {
    String(String),
    Integer(i32),
    Null,
}

impl std::fmt::Debug for PropValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropValue::String(s) => write!(f, "{:?}", hex2int(s)),
            PropValue::Integer(i) => write!(f, "{:?}", i),
            PropValue::Null => write!(f, "null"),
        }
    }
}

impl PropValue {
    pub fn from(value: f32, step: f32, size: usize) -> PropValue {
        let mut wtr = vec![];
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Metadata {
    #[serde(rename = "pt")]
    type_: String,
    #[serde(default, rename = "st")]
    step: u8,
    #[serde(rename = "mi")]
    min: Option<String>,
    #[serde(rename = "mx")]
    max: Option<String>,
}

impl std::fmt::Debug for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get_tuple())
    }
}

impl Metadata {
    pub fn step(&self) -> f32 {
        let step_base = f32::from(self.step & 0xf);
        let exp: i32 = ((self.step & 0xf0) >> 4).into();
        let step_coefficient = if exp < 8 {
            10.0_f32.powi(exp)
        } else {
            10.0_f32.powi(exp - 16)
        };
        step_base * step_coefficient
    }

    /// Returns step, min, max
    pub fn get_tuple(&self) -> (f32, Option<f32>, Option<f32>) {
        let step = self.step();
        let min = self.min.as_ref().map(|m| {
            if step != 0.0 {
                hex2int(m) as f32 * step
            } else {
                hex2int(m) as f32
            }
        });
        let max = self.max.as_ref().map(|m| {
            if step != 0.0 {
                hex2int(m) as f32 * step
            } else {
                hex2int(m) as f32
            }
        });
        (step, min, max)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn get_f32() {
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
        assert_eq!(p.get_f32(), Some(24.5));

        let json = json!({
            "pn": "root_entity_name",
            "pt": 3,
            "pv": "e_1002",
            "md": {
                "pt": "s"
            }
        });
        let p: Property = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_f32(), None);
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
        let pv = PropValue::from(p.get_f32().unwrap(), p.step(), p.size());
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
            r#"Tree { name: "e_A00D", type_: 1, children: [Item { name: "p_01", type_: 3, value: 38, metadata: (0.5, Some(-9.0), Some(39.0)) }] }"#
        );
    }
}
