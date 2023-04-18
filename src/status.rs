use crate::property::{PropValue, Property};
use crate::request::{DaikinRequest, Request};
use crate::response::DaikinResponse;
use serde_json::{from_value, Number, Value};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Debug)]
pub struct DaikinStatus {
    pub power: Option<u8>,
    pub current_temperature: Option<f32>,
    pub current_humidity: Option<f32>,
    pub current_outside_temperature: Option<f32>,
    pub mode: Option<Mode>,
    pub target_cooling_temperature: Option<f32>,
    pub target_heating_temperature: Option<f32>,
    pub target_automatic_temperature: Option<f32>,
    pub wind_speed: Option<WindSpeed>,
    pub automode_wind_speed: Option<AutoModeWindSpeed>,
    pub vertical_wind_direction: Option<VerticalDirection>,
    pub horizontal_wind_direction: Option<HorizontalDirection>,
    pub meta: Metadata,
}

#[derive(Clone, Copy, Debug)]
pub struct Meta {
    pub step: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub digits: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub power: Meta,
    pub mode: Meta,
    pub target_cooling_temperature: Meta,
    pub target_heating_temperature: Meta,
    pub target_automatic_temperature: Meta,
    pub wind_speed: Meta,
    pub automode_wind_speed: Meta,
    pub vertical_wind_direction: Meta,
    pub horizontal_wind_direction: Meta,
}

impl From<DaikinResponse> for DaikinStatus {
    fn from(response: DaikinResponse) -> Self {
        DaikinStatus {
            power: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 -> u8),
            current_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_01 -> f32),
            current_humidity: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_02 -> f32),
            current_outside_temperature: get_prop!(response."/dsiot/edge/adr_0200.dgc_status".e_1003.e_A00D.p_01 -> f32),
            mode: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 -> Value),
            target_cooling_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 -> f32),
            target_heating_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 -> f32),
            target_automatic_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F -> f32),
            wind_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_09 -> Value),
            automode_wind_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_26 -> Value),
            vertical_wind_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_05 -> Value),
            horizontal_wind_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_06 -> Value),
            meta: Metadata {
                power: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 -> meta_size),
                mode: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 -> meta_size),
                target_cooling_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 -> meta_size),
                target_heating_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 -> meta_size),
                target_automatic_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F -> meta_size),
                wind_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_09 -> meta_size),
                automode_wind_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_26 -> meta_size),
                vertical_wind_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_05 -> meta_size),
                horizontal_wind_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_06 -> meta_size),
            },
        }
    }
}

impl Into<DaikinRequest> for DaikinStatus {
    fn into(self) -> DaikinRequest {
        let mut req = DaikinRequest { requests: vec![] };

        if let Some(value) = self.power {
            let pv = PropValue::from(value as f32, self.meta.power.step, self.meta.power.digits);
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 = pv);
        };

        if let Some(value) = self.mode {
            let pv = PropValue::from(
                value as u8 as f32,
                self.meta.mode.step,
                self.meta.mode.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 = pv);
        };

        if let Some(value) = self.target_cooling_temperature {
            let pv = PropValue::from(
                value,
                self.meta.target_cooling_temperature.step,
                self.meta.target_cooling_temperature.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 = pv);
        };

        if let Some(value) = self.target_heating_temperature {
            let pv = PropValue::from(
                value,
                self.meta.target_heating_temperature.step,
                self.meta.target_heating_temperature.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 = pv);
        };

        if let Some(value) = self.target_automatic_temperature {
            let pv = PropValue::from(
                value,
                self.meta.target_automatic_temperature.step,
                self.meta.target_automatic_temperature.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F = pv);
        };

        if let Some(value) = self.wind_speed {
            let pv = PropValue::from(
                value as u8 as f32,
                self.meta.wind_speed.step,
                self.meta.wind_speed.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_09 = pv)
        };

        if let Some(value) = self.automode_wind_speed {
            let pv = PropValue::from(
                value as u8 as f32,
                self.meta.automode_wind_speed.step,
                self.meta.automode_wind_speed.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_26 = pv)
        };

        if let Some(value) = self.vertical_wind_direction {
            let pv = PropValue::from(
                value as u8 as f32,
                self.meta.vertical_wind_direction.step,
                self.meta.vertical_wind_direction.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_05 = pv)
        };

        if let Some(value) = self.horizontal_wind_direction {
            let pv = PropValue::from(
                value as u8 as f32,
                self.meta.horizontal_wind_direction.step,
                self.meta.horizontal_wind_direction.digits,
            );
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_06 = pv)
        };

        req
    }
}

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

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum AutoModeWindSpeed {
    Silent = 0x0B,
    Auto = 0x0A,

    Unknown = 0xFF,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let status: DaikinStatus = res.into();

        assert_eq!(status.power, Some(0));
        assert_eq!(status.current_temperature, Some(20.0));
        assert_eq!(status.current_humidity, Some(50.0));
        assert_eq!(status.current_outside_temperature, Some(19.0));
        assert_eq!(status.mode, Some(Mode::Cooling));
        assert_eq!(status.target_cooling_temperature, Some(24.5));
        assert_eq!(status.target_heating_temperature, Some(25.0));
        assert_eq!(status.target_automatic_temperature, Some(0.0));
        assert_eq!(status.wind_speed, Some(WindSpeed::Auto));
        assert_eq!(status.automode_wind_speed, Some(AutoModeWindSpeed::Auto));
        assert_eq!(
            status.vertical_wind_direction,
            Some(VerticalDirection::Auto)
        );
        assert_eq!(
            status.horizontal_wind_direction,
            Some(HorizontalDirection::Auto)
        );
    }

    #[test]
    fn setter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let mut status: DaikinStatus = res.into();

        status.power = Some(1);
        status.mode = Some(Mode::Cooling);
        status.target_cooling_temperature = Some(24.5);
        status.target_heating_temperature = Some(25.0);
        status.target_automatic_temperature = Some(0.0);
        status.automode_wind_speed = Some(AutoModeWindSpeed::Silent);
        status.wind_speed = Some(WindSpeed::Lev4);
        status.vertical_wind_direction = Some(VerticalDirection::BottomMost);
        status.horizontal_wind_direction = Some(HorizontalDirection::RightCenter);

        let req: DaikinRequest = status.into();
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            serde_json::from_str::<serde_json::Value>(include_str!("./fixtures/update.json"))
                .unwrap()
        );
    }

    #[test]
    fn debug_display() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let status: DaikinStatus = res.into();

        assert_eq!(
            format!("{:?}", status),
            r#"DaikinStatus { power: Some(0), current_temperature: Some(20.0), current_humidity: Some(50.0), current_outside_temperature: Some(19.0), mode: Some(Cooling), target_cooling_temperature: Some(24.5), target_heating_temperature: Some(25.0), target_automatic_temperature: Some(0.0), wind_speed: Some(Auto), automode_wind_speed: Some(Auto), vertical_wind_direction: Some(Auto), horizontal_wind_direction: Some(Auto), meta: Metadata { power: Meta { step: 1.0, min: Some(0.0), max: Some(1.0), digits: 2 }, mode: Meta { step: 0.0, min: None, max: Some(47.0), digits: 4 }, target_cooling_temperature: Meta { step: 0.5, min: Some(18.0), max: Some(32.0), digits: 2 }, target_heating_temperature: Meta { step: 0.5, min: Some(14.0), max: Some(30.0), digits: 2 }, target_automatic_temperature: Meta { step: 0.5, min: Some(-5.0), max: Some(5.0), digits: 2 }, wind_speed: Meta { step: 0.0, min: None, max: Some(3320.0), digits: 4 }, automode_wind_speed: Meta { step: 0.0, min: None, max: Some(3072.0), digits: 4 }, vertical_wind_direction: Meta { step: 0.0, min: None, max: Some(8486975.0), digits: 8 }, horizontal_wind_direction: Meta { step: 0.0, min: None, max: Some(98813.0), digits: 6 } } }"#
        );
    }
}
