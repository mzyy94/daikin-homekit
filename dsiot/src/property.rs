use core::f32;
use core::ops::RangeInclusive;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Item<T: Sized + DeserializeOwned + Into<f32> = f32> {
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

impl<T: Sized + DeserializeOwned + Into<f32>> std::fmt::Debug for Item<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{ name: {:?}, ", self.name)?;
        match &self.metadata {
            Metadata::Binary(Binary::Step(..)) => {
                if let Some(v) = self.get_f32() {
                    write!(f, "value: {v:?}")
                } else {
                    write!(f, "value: {:?}", self.value)
                }
            }
            Metadata::Binary(Binary::String { .. }) => {
                write!(f, "value: {:?}", self.get_string())
            }
            _ => {
                write!(f, "value: {:?}", self.value)
            }
        }?;
        write!(f, ", metadata: {:?} }}", self.metadata)
    }
}

fn hex2int(hex: &str) -> i32 {
    let size = hex.len();
    if size > 8 || size % 2 != 0 {
        return 0;
    }
    let mut bytes = [0u8; 4];
    if hex::decode_to_slice(hex, &mut bytes[0..size / 2]).is_err() {
        return 0;
    }
    match size {
        2 => i8::from_le_bytes([bytes[0]]) as i32,
        4 => i16::from_le_bytes([bytes[0], bytes[1]]) as i32,
        6 | 8 => i32::from_le_bytes(bytes),
        _ => 0,
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn hex2int_test() {
        let hex = "18".to_string();
        let result = hex2int(&hex);
        assert_eq!(result, 24);
        let hex = "eeff".to_string();
        let result = hex2int(&hex);
        assert_eq!(result, -18);
        let hex = "12345600".to_string();
        let result = hex2int(&hex);
        assert_eq!(result, 5649426);
    }
}

impl Property {
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

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Property> {
        match self {
            Property::Tree { children, .. } => children.iter_mut().find(|p| match p {
                Property::Tree { name: n, .. } => name == n,
                Property::Node(Item { name: n, .. }) => name == n,
            }),
            _ => None,
        }
    }

    pub fn push(&mut self, property: Property) -> Option<&mut Property> {
        match self {
            Property::Tree { children, .. } => {
                children.push(property);
                children.last_mut()
            }
            _ => None,
        }
    }
}

impl<T: Sized + DeserializeOwned + Into<f32>> Item<T> {
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

    pub fn set_value(&mut self, value: T) {
        match &self.metadata {
            Metadata::Integer => self.value = PropValue::Integer(value.into() as i32),
            Metadata::String => {} // String metadata does not support numeric value conversion
            Metadata::Binary(Binary::Step(step)) => {
                let value = (value.into() / step.step()) as i64;
                let bytes = value.to_le_bytes();
                self.value = PropValue::String(hex::encode(&bytes[..(step.max.len() / 2)]));
            }
            Metadata::Binary(Binary::Enum { max }) => {
                let value = value.into() as i64;
                let bytes = value.to_le_bytes();
                self.value = PropValue::String(hex::encode(&bytes[..(max.len() / 2)]));
            }
            _ => self.value = PropValue::Null,
        }
    }

    pub fn get_int(&self) -> Option<i32> {
        match self {
            Item {
                value: PropValue::Integer(pv),
                metadata: Metadata::Integer,
                ..
            } => Some(*pv),
            _ => None,
        }
    }

    pub fn get_enum(&self) -> Option<T> {
        match self {
            Item {
                value: PropValue::String(pv),
                metadata: Metadata::Binary(Binary::Enum { .. }),
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
                if matches!(md, Metadata::String) {
                    Some(pv.clone())
                } else if matches!(md, Metadata::Binary(Binary::String { .. })) {
                    hex::decode(pv)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                        .map(|s| s.chars().rev().collect())
                        .map(|s: String| s.trim_end_matches('\0').to_string())
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
    StringList(Vec<String>),
    IntegerList(Vec<i32>),
    Object(serde_json::Value),
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
    #[serde(rename = "o")]
    Object,
    #[serde(rename = "l<s>")]
    StringList,
    #[serde(rename = "l<i>")]
    IntegerList,
    Undefined,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum Binary {
    Step(BinaryStep),
    Enum {
        #[serde(rename = "mx")]
        max: String,
    },
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
        let p: Item = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_f32(), Some(24.5));

        let json = json!({
            "pn": "root_entity_name",
            "pt": 3,
            "pv": "e_1002",
            "md": {
                "pt": "s"
            }
        });
        let p: Item = serde_json::from_value(json).expect("Invalid JSON structure.");
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
        let p: Item = serde_json::from_value(json).expect("Invalid JSON structure.");
        assert_eq!(p.get_string(), Some("e_1002".into()));

        let json = json!({
            "pn": "data_model_code",
            "pt": 3,
            "pv": 26,
            "md": {
                "pt": "i"
            }
        });
        let p: Item = serde_json::from_value(json).expect("Invalid JSON structure.");
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
        let item: Item = serde_json::from_value(json).expect("Invalid JSON structure.");
        let pv = item.value;
        let expect = PropValue::String("2600".into());

        assert_eq!(pv, expect);
    }

    #[test]
    fn find_push() {
        let mut tree = Property::new_tree("root");
        let _child = tree.push(Property::new_tree("child")).unwrap();
        assert!(matches!(tree.find("child"), Some(Property::Tree { .. })));
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
            format!("{p:?}"),
            r#"Tree { name: "e_A00D", children: [Node(Item { name: "p_01", value: 19.0, metadata: Binary(Step(BinaryStep { range: -9.0..=39.0, step: 0.5 })) })] }"#
        );
    }
}
