use crate::property::{PropValue, Property};
use crate::request::{DaikinRequest, Request};
use crate::response::DaikinResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Fan = 0,
    Heating = 1,
    Cooling = 2,
    Auto = 3,
    Dehumidify = 5,

    Unknown = 255,
}

impl std::convert::From<u8> for Mode {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::Fan,
            1 => Self::Heating,
            2 => Self::Cooling,
            3 => Self::Auto,
            5 => Self::Dehumidify,
            _ => Self::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct DaikinStatus {
    pub power: Option<u8>,
    pub current_temperature: Option<f32>,
    pub current_humidity: Option<f32>,
    pub current_outside_temperature: Option<f32>,
    pub mode: Option<Mode>,
    pub target_cooling_temperature: Option<f32>,
    pub target_heating_temperature: Option<f32>,
    pub target_automatic_temperature: Option<f32>,
    meta: Meta,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Meta {
    power: ((f32, Option<f32>, Option<f32>), usize),
    mode: ((f32, Option<f32>, Option<f32>), usize),
    target_cooling_temperature: ((f32, Option<f32>, Option<f32>), usize),
    target_heating_temperature: ((f32, Option<f32>, Option<f32>), usize),
    target_automatic_temperature: ((f32, Option<f32>, Option<f32>), usize),
}

impl DaikinStatus {
    pub fn new(response: DaikinResponse) -> Self {
        DaikinStatus {
            power: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 -> u8),
            current_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_01 -> f32),
            current_humidity: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_02 -> f32),
            current_outside_temperature: get_prop!(response."/dsiot/edge/adr_0200.dgc_status".e_1003.e_A00D.p_01 -> f32),
            mode: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 -> u8)
                .map(|v| v.into()),
            target_cooling_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 -> f32),
            target_heating_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 -> f32),
            target_automatic_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F -> f32),
            meta: Meta {
                power: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 -> meta_size),
                mode: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 -> meta_size),
                target_cooling_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 -> meta_size),
                target_heating_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 -> meta_size),
                target_automatic_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F -> meta_size),
            },
        }
    }

    pub fn build_request(&self) -> DaikinRequest {
        let mut req = DaikinRequest { requests: vec![] };

        if let Some(value) = self.power {
            let pv = PropValue::from(value as f32, self.meta.power.0 .0, self.meta.power.1);
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 = pv);
        };

        if let Some(value) = self.mode {
            let pv = PropValue::from(value as u8 as f32, self.meta.mode.0 .0, self.meta.mode.1);
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 = pv);
        };

        if let Some(value) = self.target_cooling_temperature {
            let pv = PropValue::from(
                value,
                self.meta.target_cooling_temperature.0 .0,
                self.meta.target_cooling_temperature.1,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 = pv);
        };

        if let Some(value) = self.target_heating_temperature {
            let pv = PropValue::from(
                value,
                self.meta.target_heating_temperature.0 .0,
                self.meta.target_heating_temperature.1,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 = pv);
        };

        if let Some(value) = self.target_automatic_temperature {
            let pv = PropValue::from(
                value,
                self.meta.target_automatic_temperature.0 .0,
                self.meta.target_automatic_temperature.1,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F = pv);
        };

        req
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let status = DaikinStatus::new(res);

        assert_eq!(status.power, Some(0));
        assert_eq!(status.current_temperature, Some(20.0));
        assert_eq!(status.current_humidity, Some(50.0));
        assert_eq!(status.current_outside_temperature, Some(19.0));
        assert_eq!(status.mode, Some(Mode::Cooling));
        assert_eq!(status.target_cooling_temperature, Some(24.5));
        assert_eq!(status.target_heating_temperature, Some(25.0));
        assert_eq!(status.target_automatic_temperature, Some(0.0));
    }

    #[test]
    fn setter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let mut status = DaikinStatus::new(res);

        status.power = Some(1);
        status.mode = Some(Mode::Cooling);
        status.target_cooling_temperature = Some(24.5);
        status.target_heating_temperature = Some(25.0);
        status.target_automatic_temperature = Some(0.0);

        let req = status.build_request();
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            serde_json::from_str::<serde_json::Value>(include_str!("./fixtures/update.json"))
                .unwrap()
        );
    }
}
