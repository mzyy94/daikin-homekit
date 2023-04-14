use crate::property::Property;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DaikinResponse {
    pub responses: Vec<Response>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Response {
    pub fr: String,           // from
    pub pc: Option<Property>, // content
    pub rsc: u32,             // response status code
}

macro_rules! get_child_prop {
    ({ $vopt:expr }) => {
        $vopt.ok_or(Error::NoProperty)
    };
    ({ $vopt:expr } -> meta_size) => {
        ($vopt.map(|v| v.meta()).unwrap_or_default(), $vopt.map(|v| v.size()).unwrap_or_default())
    };
    ({ $vopt:expr } -> f32) => {
        $vopt.and_then(|v| v.get_f32())
    };
    ({ $vopt:expr } -> u8) => {
        $vopt.and_then(|v| v.get_f32()).map(|v| v as u8)
    };
    ({ $vopt:expr } -> Value) => {
        from_value($vopt.and_then(|v| v.get_f32()).map(|v| Value::Number(Number::from(v as u8))).unwrap_or_default()).unwrap_or_default()
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
            { $v.responses.iter().find(|&r| r.fr == $key).and_then(|r| r.pc.as_ref()) } $($rest)*
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
            r#"Ok(Item { name: "p_03", pv: 5.6, meta: (0.1, Some(0.0), Some(25.5)) })"#
        );

        let p = get_prop!(status."/hoge".fuga.piyo);
        assert_eq!(format!("{:?}", p), r#"Err(NoProperty)"#);
    }
}
