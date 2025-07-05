use crate::property::Property;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct DaikinRequest {
    pub requests: Vec<Request>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Request {
    pub op: u8,
    pub pc: Property,
    pub to: String,
}

#[cfg(test)]
mod tests {
    use crate::property::{Item, PropValue};

    use super::*;

    #[derive(Serialize)]
    struct TestDaikinStatus {
        requests: Vec<Request>,
    }

    #[test]
    fn set_prop() {
        let mut status = TestDaikinStatus { requests: vec![] };

        let item: Item<f32> = Item {
            name: "p_03".into(),
            value: PropValue::String("3800".into()),
            metadata: crate::property::Metadata::Undefined,
            phantom: std::marker::PhantomData,
        };
        set_prop!(&mut status."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A001.p_03 = item);
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            r#"{"requests":[{"op":3,"pc":{"pn":"dgc_status","pch":[{"pn":"e_1002","pch":[{"pn":"e_A001","pch":[{"pn":"p_03","pv":"3800"}]}]}]},"to":"/dsiot/edge/adr_0100.dgc_status"}]}"#
        );
    }
}
