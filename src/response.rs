use crate::property::Property;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub fr: String,   // from
    pub pc: Property, // content
    pub rsc: u32,     // ??
}

macro_rules! get_child_prop {
    ({ $vopt:expr }) => {
        $vopt
    };
    ({ $vopt:expr } -> f64) => {
        $vopt.and_then(|v| v.get_f64())
    };
    ({ $vopt:expr } -> u8) => {
        $vopt.and_then(|v| v.get_f64()).map(|v| v as u8)
    };
    ({ $vopt:expr } -> bool) => {
        $vopt.and_then(|v| v.get_f64()).map(|v| v  == 1.0)
    };
    ({ $vopt:expr } -> str) => {
        $vopt.and_then(|v| v.get_string())
    };
    ({ $vopt:expr } . $key:ident $($rest:tt)*) => {
        get_child_prop!(
            { $vopt.and_then(|v| v.find(stringify!($key))) } $($rest)*
        )
    };
}

macro_rules! get_prop {
    ($v:tt . $key:literal $($rest:tt)*) => {
        get_child_prop!(
            { $v.responses.iter().find(|&r| r.fr == $key).map(|r| &r.pc) }  $($rest)*
        )
    };
}
