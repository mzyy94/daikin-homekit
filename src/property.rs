use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use core::f32;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::Cursor, ops::RangeInclusive};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Property {
    Tree {
        #[serde(rename = "pn")]
        name: String,
        // #[serde(skip_serializing, rename = "pt")]
        // type_: u8, // 1
        #[serde(rename = "pch")]
        children: Vec<Property>,
    },
    Node(Item),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item<T: Sized + DeserializeOwned = f32> {
    #[serde(rename = "pn")]
    pub name: String,
    // #[serde(skip_serializing, rename = "pt")]
    // type_: u8, // 2, 3
    #[serde(rename = "pv")]
    pub value: PropValue,
    #[serde(skip_serializing, rename = "md")]
    pub metadata: Metadata,
    #[serde(skip)]
    pub phantom: std::marker::PhantomData<fn() -> T>,
}

fn hex2int(hex: &String) -> i32 {
    let size = hex.len();
    if size > 8 {
        return 0;
    }
    let mut bytes = vec![0u8; size / 2];
    hex::decode_to_slice(hex, &mut bytes as &mut [u8]).ok();
    let mut rdr = Cursor::new(bytes);

    rdr.read_int::<LittleEndian>(size / 2)
        .unwrap()
        .try_into()
        .unwrap_or(0)
}

impl Property {
    pub fn new(name: &str, value: PropValue) -> Property {
        Property::Node(Item {
            name: name.to_string(),
            value: value,
            metadata: Metadata::Undefined,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn new_tree(name: &str) -> Property {
        Property::Tree {
            name: name.to_string(),
            children: vec![],
        }
    }

    pub fn find(&self, name: &str) -> Option<&Property> {
        match self {
            Property::Tree { children, .. } => children.iter().find(|p| match p {
                Property::Tree { name: n, .. } => name == n,
                Property::Node(Item { name: n, .. }) => name == n,
            }),
            _ => None,
        }
    }
}

impl<T: Sized + DeserializeOwned> Item<T> {
    pub fn meta(&self) -> (f32, Option<f32>, Option<f32>) {
        match self {
            Item {
                metadata: Metadata::Binary(Binary::Step(step)),
                ..
            } => (
                step.step(),
                Some(*step.range().start()),
                Some(*step.range().end()),
            ),
            Item {
                metadata: Metadata::Binary(Binary::Enum(enum_)),
                ..
            } => (0.0, Some(f32::NAN), Some(hex2int(&enum_.max) as f32)),
            _ => (0.0, None, None),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Item {
                value: PropValue::String(str),
                ..
            } => str.len(),
            _ => 0,
        }
    }

    pub fn get_f32(&self) -> Option<f32> {
        match self {
            Item {
                value: PropValue::String(pv),
                metadata: Metadata::Binary(Binary::Step(step)),
                ..
            } => {
                let value = hex2int(pv) as f32;
                let step = step.step();
                Some(value * step)
            }
            _ => None,
        }
    }

    pub fn get_int(&self) -> Option<i32> {
        match self {
            Item {
                value: PropValue::Integer(pv),
                metadata: Metadata::Integer {},
                ..
            } => Some(*pv),
            _ => None,
        }
    }

    pub fn get_enum(&self) -> Option<T> {
        match self {
            Item {
                value: PropValue::String(pv),
                metadata: Metadata::Binary(Binary::Enum(..)),
                ..
            } => {
                let value = hex2int(pv);
                serde_json::from_value(serde_json::Value::Number(serde_json::Number::from(value)))
                    .ok()
            }
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            Item {
                value: PropValue::String(pv),
                metadata: md,
                ..
            } => {
                if matches!(md, Metadata::String {}) {
                    Some(String::from(pv))
                } else if matches!(md, Metadata::Binary(Binary::String { .. })) {
                    todo!() // decode hex string
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum PropValue {
    String(String),
    Integer(i32),
    Null,
}

impl PropValue {
    pub fn from(value: f32, step: f32, size: usize) -> PropValue {
        if value.is_nan() || size > 8 || size < 2 {
            return PropValue::Null;
        }
        let mut wtr = vec![];
        let value = if step == 0.0 {
            value as i64
        } else {
            (value / step) as i64
        };
        wtr.write_int::<LittleEndian>(value, size / 2).unwrap();
        PropValue::String(hex::encode(wtr))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "pt")]
pub enum Metadata {
    #[serde(rename = "i")]
    Integer,
    #[serde(rename = "s")]
    String,
    #[serde(rename = "b")]
    Binary(Binary),
    Undefined,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum Binary {
    Step(BinaryStep),
    Enum(BinaryEnum),
    String {},
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct BinaryStep {
    #[serde(rename = "st")]
    pub step: u8,
    #[serde(rename = "mi")]
    pub min: String,
    #[serde(rename = "mx")]
    pub max: String,
}

impl BinaryStep {
    pub fn step(&self) -> f32 {
        let step = self.step;
        let step_base = f32::from(step & 0xf);
        let exp: i8 = (step & 0xf0) as i8 >> 4;
        let step_coefficient = 10.0_f32.powi(exp as i32);
        step_base * step_coefficient
    }

    pub fn range(&self) -> RangeInclusive<f32> {
        let BinaryStep { min, max, step } = self;
        let step = if *step == 0 { 1.0 } else { self.step() };
        let min_value = hex2int(min) as f32 * step;
        let max_value = hex2int(max) as f32 * step;
        RangeInclusive::new(min_value, max_value)
    }
}

impl std::fmt::Debug for BinaryStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BinaryStep {{ range: {:?}, step: {} }}",
            self.range(),
            self.step(),
        )
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BinaryEnum {
    // #[serde(rename = "st")]
    // step: u8, // 0
    #[serde(rename = "mx")]
    pub max: String,
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
        let pv = PropValue::from(p.get_f32().unwrap(), p.meta().0, p.size());
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
            r#"Tree { name: "e_A00D", children: [Item { name: "p_01", value: String("2600"), metadata: Binary(Step(BinaryStep { step: 245, min: "EEFF", max: "4E00" })) }] }"#
        );
    }
}
