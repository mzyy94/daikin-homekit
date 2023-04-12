use crate::property::Property;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Response {
    pub fr: String,   // from
    pub pc: Property, // content
    pub rsc: u32,     // response code
}

macro_rules! get_child_prop {
    ({ $vopt:expr }) => {
        $vopt.ok_or(Error::NoProperty)
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

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use super::*;

    #[derive(Deserialize)]
    struct TestDaikinStatus {
        responses: Vec<Response>,
    }

    #[test]
    fn get_prop() {
        let status: TestDaikinStatus = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");

        let p = get_prop!(status."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A001.p_03);
        assert_eq!(
            format!("{:?}", p),
            r#"Ok(Item { name: "p_03", pv: 5.6000000000000005 })"#
        );

        let p = get_prop!(status."/hoge".fuga.piyo);
        assert_eq!(format!("{:?}", p), r#"Err(NoProperty)"#);
    }
}
