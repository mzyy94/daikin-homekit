//! Wind/airflow control type definitions.

use serde_repr::{Deserialize_repr, Serialize_repr};

/// Fan speed setting.
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum WindSpeed {
    Silent = 0x0B,
    Lev1 = 0x03,
    Lev2 = 0x04,
    Lev3 = 0x05,
    Lev4 = 0x06,
    Lev5 = 0x07,
    Auto = 0x0A,

    Unknown = 0xFF,
}

impl From<WindSpeed> for f32 {
    fn from(val: WindSpeed) -> Self {
        val as u8 as f32
    }
}

/// Fan speed setting for auto mode.
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum AutoModeWindSpeed {
    Silent = 0x0B,
    Auto = 0x0A,

    Unknown = 0xFF,
}

impl From<AutoModeWindSpeed> for f32 {
    fn from(val: AutoModeWindSpeed) -> Self {
        val as u8 as f32
    }
}

/// Vertical air direction.
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum VerticalDirection {
    TopMost = 0x01,
    Top = 0x02,
    Center = 0x03,
    Bottom = 0x04,
    BottomMost = 0x05,

    Swing = 0x0F,
    Auto = 0x10,

    Nice = 0x17,

    Unknown = 0xFF,
}

impl From<VerticalDirection> for f32 {
    fn from(val: VerticalDirection) -> Self {
        val as u8 as f32
    }
}

/// Horizontal air direction.
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HorizontalDirection {
    LeftMost = 0x02,
    Left = 0x03,
    LeftCenter = 0x04,
    Center = 0x05,
    RightCenter = 0x06,
    Right = 0x07,
    RightMost = 0x08,

    Swing = 0x0F,
    Auto = 0x10,

    Unknown = 0xFF,
}

impl From<HorizontalDirection> for f32 {
    fn from(val: HorizontalDirection) -> Self {
        val as u8 as f32
    }
}
