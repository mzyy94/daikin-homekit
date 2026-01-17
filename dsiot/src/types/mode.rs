//! Operating mode definitions.

use serde_repr::{Deserialize_repr, Serialize_repr};

/// HVAC operating mode.
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Mode {
    Fan = 0,
    Heating = 1,
    Cooling = 2,
    Auto = 3,
    Dehumidify = 5,

    Unknown = 255,
}

impl From<Mode> for f32 {
    fn from(val: Mode) -> Self {
        val as u8 as f32
    }
}
