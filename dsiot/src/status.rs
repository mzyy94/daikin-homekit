use crate::request::DaikinRequest;
use crate::response::DaikinResponse;
use crate::{
    property::{Item, Property},
    request::Request,
};
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
        let mut prop = Property::new_tree("dgc_status");

        set_child_prop!({ prop }.e_1002.e_A002.p_01 = self.power);
        set_child_prop!({ prop }.e_1002.e_3001.p_01 = self.mode);
        set_child_prop!({ prop }.e_1002.e_3001.p_02 = self.target_cooling_temperature);
        set_child_prop!({ prop }.e_1002.e_3001.p_03 = self.target_heating_temperature);
        set_child_prop!({ prop }.e_1002.e_3001.p_1F = self.target_automatic_temperature);
        set_child_prop!({ prop }.e_1002.e_3001.p_09 = self.wind_speed);
        set_child_prop!({ prop }.e_1002.e_3001.p_26 = self.automode_wind_speed);
        set_child_prop!({ prop }.e_1002.e_3001.p_05 = self.vertical_wind_direction);
        set_child_prop!({ prop }.e_1002.e_3001.p_06 = self.horizontal_wind_direction);

        DaikinRequest {
            requests: vec![Request {
                op: 3,
                pc: prop,
                to: "/dsiot/edge/adr_0100.dgc_status".into(),
            }],
        }
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

impl From<Mode> for f32 {
    fn from(val: Mode) -> Self {
        val as u8 as f32
    }
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

impl From<WindSpeed> for f32 {
    fn from(val: WindSpeed) -> Self {
        val as u8 as f32
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, Clone)]
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

impl From<VerticalDirection> for f32 {
    fn from(val: VerticalDirection) -> Self {
        val as u8 as f32
    }
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

impl From<HorizontalDirection> for f32 {
    fn from(val: HorizontalDirection) -> Self {
        val as u8 as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let status: DaikinStatus = res.into();

        assert_eq!(status.power.get_f32(), Some(0.0));
        assert_eq!(status.current_temperature.get_f32(), Some(20.0));
        assert_eq!(status.current_humidity.get_f32(), Some(50.0));
        assert_eq!(status.current_outside_temperature.get_f32(), Some(19.0));
        assert_eq!(status.mode.get_enum(), Some(Mode::Cooling));
        assert_eq!(status.target_cooling_temperature.get_f32(), Some(24.5));
        assert_eq!(status.target_heating_temperature.get_f32(), Some(25.0));
        assert_eq!(status.target_automatic_temperature.get_f32(), Some(0.0));
        assert_eq!(status.wind_speed.get_enum(), Some(WindSpeed::Auto));
        assert_eq!(
            status.automode_wind_speed.get_enum(),
            Some(AutoModeWindSpeed::Auto)
        );
        assert_eq!(
            status.vertical_wind_direction.get_enum(),
            Some(VerticalDirection::Auto)
        );
        assert_eq!(
            status.horizontal_wind_direction.get_enum(),
            Some(HorizontalDirection::Auto)
        );
    }

    #[test]
    fn setter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let mut status: DaikinStatus = res.into();

        status.power.set_value(1.0);
        status.mode.set_value(Mode::Cooling);
        status.target_cooling_temperature.set_value(24.5);
        status.target_heating_temperature.set_value(25.0);
        status.target_automatic_temperature.set_value(0.0);
        status
            .automode_wind_speed
            .set_value(AutoModeWindSpeed::Silent);
        status.wind_speed.set_value(WindSpeed::Lev4);
        status
            .vertical_wind_direction
            .set_value(VerticalDirection::BottomMost);
        status
            .horizontal_wind_direction
            .set_value(HorizontalDirection::RightCenter);

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
            r#"DaikinStatus { power: Item { name: "p_01", value: 0.0, metadata: Binary(Step(BinaryStep { range: 0.0..=1.0, step: 1 })) }, current_temperature: Item { name: "p_01", value: 20.0, metadata: Binary(Step(BinaryStep { range: -9.0..=39.0, step: 1 })) }, current_humidity: Item { name: "p_02", value: 50.0, metadata: Binary(Step(BinaryStep { range: 25.0..=85.0, step: 1 })) }, current_outside_temperature: Item { name: "p_01", value: 19.0, metadata: Binary(Step(BinaryStep { range: -9.0..=39.0, step: 0.5 })) }, mode: Item { name: "p_01", value: String("0200"), metadata: Binary(Enum { max: "2F00" }) }, target_cooling_temperature: Item { name: "p_02", value: 24.5, metadata: Binary(Step(BinaryStep { range: 18.0..=32.0, step: 0.5 })) }, target_heating_temperature: Item { name: "p_03", value: 25.0, metadata: Binary(Step(BinaryStep { range: 14.0..=30.0, step: 0.5 })) }, target_automatic_temperature: Item { name: "p_1F", value: 0.0, metadata: Binary(Step(BinaryStep { range: -5.0..=5.0, step: 0.5 })) }, wind_speed: Item { name: "p_09", value: String("0A00"), metadata: Binary(Enum { max: "F80C" }) }, automode_wind_speed: Item { name: "p_26", value: String("0A00"), metadata: Binary(Enum { max: "000C" }) }, vertical_wind_direction: Item { name: "p_05", value: String("10000000"), metadata: Binary(Enum { max: "3F808100" }) }, horizontal_wind_direction: Item { name: "p_06", value: String("100000"), metadata: Binary(Enum { max: "FD8101" }) } }"#
        );
    }
}
