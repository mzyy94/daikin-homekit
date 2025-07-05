use crate::property::Item;
use crate::request::DaikinRequest;
use crate::response::DaikinResponse;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug)]
pub struct DaikinStatus {
    pub power: Item<f32>,
    pub current_temperature: Item<f32>,
    pub current_humidity: Item<f32>,
    pub current_outside_temperature: Item<f32>,
    pub mode: Item<Mode>,
    pub target_cooling_temperature: Item<f32>,
    pub target_heating_temperature: Item<f32>,
    pub target_automatic_temperature: Item<f32>,
    pub wind_speed: Item<WindSpeed>,
    pub automode_wind_speed: Item<AutoModeWindSpeed>,
    pub vertical_wind_direction: Item<VerticalDirection>,
    pub horizontal_wind_direction: Item<HorizontalDirection>,
}

impl From<DaikinResponse> for DaikinStatus {
    fn from(response: DaikinResponse) -> Self {
        DaikinStatus {
            power: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01),
            current_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_01),
            current_humidity: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_02),
            current_outside_temperature: get_prop!(response."/dsiot/edge/adr_0200.dgc_status".e_1003.e_A00D.p_01),
            mode: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01),
            target_cooling_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02),
            target_heating_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03),
            target_automatic_temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F),
            wind_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_09),
            automode_wind_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_26),
            vertical_wind_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_05),
            horizontal_wind_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_06),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<DaikinRequest> for DaikinStatus {
    fn into(self) -> DaikinRequest {
        let mut req = DaikinRequest { requests: vec![] };

        if let Item { value: pv, .. } = self.power {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01 = pv);
        };

        if let Item { value: pv, .. } = self.mode {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01 = pv);
        };

        if let Item { value: pv, .. } = self.target_cooling_temperature {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02 = pv);
        };

        if let Item { value: pv, .. } = self.target_heating_temperature {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03 = pv);
        };

        if let Item { value: pv, .. } = self.target_automatic_temperature {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F = pv);
        };

        if let Item { value: pv, .. } = self.wind_speed {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_09 = pv);
        };

        if let Item { value: pv, .. } = self.automode_wind_speed {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_26 = pv);
        };

        if let Item { value: pv, .. } = self.vertical_wind_direction {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_05 = pv);
        };

        if let Item { value: pv, .. } = self.horizontal_wind_direction {
            set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_06 = pv);
        };

        req
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum Mode {
    Fan = 0,
    Heating = 1,
    Cooling = 2,
    Auto = 3,
    Dehumidify = 5,

    Unknown = 255,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone)]
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

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
pub enum AutoModeWindSpeed {
    Silent = 0x0B,
    Auto = 0x0A,

    Unknown = 0xFF,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone)]
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

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone)]
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
        let json = serde_json::to_value(req).unwrap();
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
            r#"DaikinStatus { power: Some(0), current_temperature: Some(20.0), current_humidity: Some(50.0), current_outside_temperature: Some(19.0), mode: Some(Cooling), target_cooling_temperature: Some(24.5), target_heating_temperature: Some(25.0), target_automatic_temperature: Some(0.0), wind_speed: Some(Auto), automode_wind_speed: Some(Auto), vertical_wind_direction: Some(Auto), horizontal_wind_direction: Some(Auto), meta: Metadata { power: Meta { step: 1.0, min: Some(0.0), max: Some(1.0), digits: 2 }, current_temperature: Meta { step: 1.0, min: Some(-9.0), max: Some(39.0), digits: 2 }, mode: Meta { step: 0.0, min: Some(NaN), max: Some(47.0), digits: 4 }, target_cooling_temperature: Meta { step: 0.5, min: Some(18.0), max: Some(32.0), digits: 2 }, target_heating_temperature: Meta { step: 0.5, min: Some(14.0), max: Some(30.0), digits: 2 }, target_automatic_temperature: Meta { step: 0.5, min: Some(-5.0), max: Some(5.0), digits: 2 }, wind_speed: Meta { step: 0.0, min: Some(NaN), max: Some(3320.0), digits: 4 }, automode_wind_speed: Meta { step: 0.0, min: Some(NaN), max: Some(3072.0), digits: 4 }, vertical_wind_direction: Meta { step: 0.0, min: Some(NaN), max: Some(8486975.0), digits: 8 }, horizontal_wind_direction: Meta { step: 0.0, min: Some(NaN), max: Some(98813.0), digits: 6 } } }"#
        );
    }
}
