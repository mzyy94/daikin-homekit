use crate::property::{PropValue, Property};
use crate::request::Request;
use crate::response::Response;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize)]
pub struct DaikinStatus {
    #[serde(skip_serializing)]
    responses: Vec<Response>,
    #[serde(skip_deserializing)]
    requests: Vec<Request>,
}

impl DaikinStatus {
    pub fn new() -> DaikinStatus {
        DaikinStatus {
            responses: vec![],
            requests: vec![],
        }
    }

    pub fn power(&self) -> Option<bool> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 -> bool)
    }

    pub fn set_power(&mut self, on: bool) {
        let val = PropValue::String(if on { "01" } else { "00" }.into());
        set_prop!(&mut self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 = val);
    }

    pub fn current_temperature(&self) -> Option<f64> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_01 -> f64)
    }

    pub fn current_humidity(&self) -> Option<f64> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_02 -> f64)
    }

    pub fn current_outside_temperature(&self) -> Option<f64> {
        get_prop!(self."/dsiot/edge/adr_0200.dgc_status".e_1003.e_A00D.p_01 -> f64)
    }

    pub fn mode(&self) -> Option<Mode> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 -> u8).map(|v| v.into())
    }

    pub fn set_mode(&mut self, mode: Mode) -> () {
        let prop = get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01).unwrap();
        let value = f64::from(mode as u8);
        let pv = PropValue::from(value, prop.step(), prop.size());
        set_prop!(&mut self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 = pv);
    }

    pub fn target_cooling_temperature(&self) -> Option<f64> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 -> f64)
    }

    pub fn set_target_cooling_temperature(&mut self, temp: f64) -> () {
        let prop = get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02).unwrap();
        let pv = PropValue::from(temp, prop.step(), prop.size());
        set_prop!(&mut self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 = pv);
    }

    pub fn target_heating_temperature(&self) -> Option<f64> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 -> f64)
    }

    pub fn set_target_heating_temperature(&mut self, temp: f64) -> () {
        let prop = get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03).unwrap();
        let pv = PropValue::from(temp, prop.step(), prop.size());
        set_prop!(&mut self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 = pv);
    }

    pub fn target_automatic_temperature(&self) -> Option<f64> {
        get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F -> f64)
    }

    pub fn set_target_automatic_temperature(&mut self, temp: f64) -> () {
        let prop = get_prop!(self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F).unwrap();
        let pv = PropValue::from(temp, prop.step(), prop.size());
        set_prop!(&mut self."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F = pv);
    }
}

impl std::fmt::Debug for DaikinStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DaikinStatus")
            .field("power", &self.power())
            .field("current_temperature", &self.current_temperature())
            .field("current_humidity", &self.current_humidity())
            .field(
                "current_outside_temperature",
                &self.current_outside_temperature(),
            )
            .field("mode", &self.mode())
            .field(
                "target_cooling_temperature",
                &self.target_cooling_temperature(),
            )
            .field(
                "target_heating_temperature",
                &self.target_heating_temperature(),
            )
            .field(
                "target_automatic_temperature",
                &self.target_automatic_temperature(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let status: DaikinStatus = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");

        assert_eq!(status.power(), Some(false));
        assert_eq!(status.current_temperature(), Some(20.0));
        assert_eq!(status.current_humidity(), Some(50.0));
        assert_eq!(status.current_outside_temperature(), Some(19.0));
        assert_eq!(status.mode(), Some(Mode::Cooling));
        assert_eq!(status.target_cooling_temperature(), Some(24.5));
        assert_eq!(status.target_heating_temperature(), Some(25.0));
        assert_eq!(status.target_automatic_temperature(), Some(0.0));
    }

    #[test]
    fn setter() {
        let mut status: DaikinStatus = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        status.set_power(true);
        status.set_mode(Mode::Cooling);
        status.set_target_cooling_temperature(24.5);
        status.set_target_heating_temperature(25.0);
        status.set_target_automatic_temperature(0.0);

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(
            json,
            serde_json::from_str::<serde_json::Value>(include_str!("./fixtures/update.json"))
                .unwrap()
        );
    }

    #[test]
    fn debug_display() {
        let status: DaikinStatus = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");

        assert_eq!(
            format!("{:?}", status),
            r#"DaikinStatus { power: Some(false), current_temperature: Some(20.0), current_humidity: Some(50.0), current_outside_temperature: Some(19.0), mode: Some(Cooling), target_cooling_temperature: Some(24.5), target_heating_temperature: Some(25.0), target_automatic_temperature: Some(0.0) }"#
        );
    }
}
