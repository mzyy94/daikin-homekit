use crate::request::DaikinRequest;
use crate::response::DaikinResponse;
use crate::{
    property::{Item, Property},
    request::Request,
};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Sensor readings from the device (read-only values).
#[derive(Clone, Debug)]
pub struct SensorReadings {
    /// Indoor temperature in Celsius.
    pub temperature: Item<f32>,
    /// Indoor humidity percentage.
    pub humidity: Item<f32>,
    /// Outdoor temperature in Celsius.
    pub outdoor_temperature: Item<f32>,
}

/// Temperature target settings for each mode.
#[derive(Clone, Debug)]
pub struct TemperatureSettings {
    /// Target temperature for cooling mode.
    pub cooling: Item<f32>,
    /// Target temperature for heating mode.
    pub heating: Item<f32>,
    /// Temperature offset for auto mode (-5 to +5).
    pub automatic: Item<f32>,
}

/// Wind/airflow control settings.
#[derive(Clone, Debug)]
pub struct WindSettings {
    /// Fan speed setting.
    pub speed: Item<WindSpeed>,
    /// Fan speed for auto mode.
    pub automode_speed: Item<AutoModeWindSpeed>,
    /// Vertical air direction.
    pub vertical_direction: Item<VerticalDirection>,
    /// Horizontal air direction.
    pub horizontal_direction: Item<HorizontalDirection>,
}

/// Complete device status containing all readable and writable properties.
#[derive(Clone, Debug)]
pub struct DaikinStatus {
    /// Power state (0.0 = off, 1.0 = on).
    pub power: Item<f32>,
    /// Operating mode.
    pub mode: Item<Mode>,
    /// Sensor readings (temperature, humidity).
    pub sensors: SensorReadings,
    /// Temperature settings for each mode.
    pub temperature: TemperatureSettings,
    /// Wind/airflow settings.
    pub wind: WindSettings,
}

impl From<DaikinResponse> for DaikinStatus {
    fn from(response: DaikinResponse) -> Self {
        DaikinStatus {
            power: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A002.p_01),
            mode: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_01),
            sensors: SensorReadings {
                temperature: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_01),
                humidity: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A00B.p_02),
                outdoor_temperature: get_prop!(response."/dsiot/edge/adr_0200.dgc_status".e_1003.e_A00D.p_01),
            },
            temperature: TemperatureSettings {
                cooling: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_02),
                heating: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_03),
                automatic: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_1F),
            },
            wind: WindSettings {
                speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_09),
                automode_speed: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_26),
                vertical_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_05),
                horizontal_direction: get_prop!(response."/dsiot/edge/adr_0100.dgc_status".e_1002.e_3001.p_06),
            },
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<DaikinRequest> for DaikinStatus {
    fn into(self) -> DaikinRequest {
        let mut prop = Property::new_tree("dgc_status");

        set_child_prop!({ prop }.e_1002.e_A002.p_01 = self.power);
        set_child_prop!({ prop }.e_1002.e_3001.p_01 = self.mode);
        set_child_prop!({ prop }.e_1002.e_3001.p_02 = self.temperature.cooling);
        set_child_prop!({ prop }.e_1002.e_3001.p_03 = self.temperature.heating);
        set_child_prop!({ prop }.e_1002.e_3001.p_1F = self.temperature.automatic);
        set_child_prop!({ prop }.e_1002.e_3001.p_09 = self.wind.speed);
        set_child_prop!({ prop }.e_1002.e_3001.p_26 = self.wind.automode_speed);
        set_child_prop!({ prop }.e_1002.e_3001.p_05 = self.wind.vertical_direction);
        set_child_prop!({ prop }.e_1002.e_3001.p_06 = self.wind.horizontal_direction);

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
        assert_eq!(status.mode.get_enum(), Some(Mode::Cooling));

        // Sensor readings
        assert_eq!(status.sensors.temperature.get_f32(), Some(20.0));
        assert_eq!(status.sensors.humidity.get_f32(), Some(50.0));
        assert_eq!(status.sensors.outdoor_temperature.get_f32(), Some(19.0));

        // Temperature settings
        assert_eq!(status.temperature.cooling.get_f32(), Some(24.5));
        assert_eq!(status.temperature.heating.get_f32(), Some(25.0));
        assert_eq!(status.temperature.automatic.get_f32(), Some(0.0));

        // Wind settings
        assert_eq!(status.wind.speed.get_enum(), Some(WindSpeed::Auto));
        assert_eq!(
            status.wind.automode_speed.get_enum(),
            Some(AutoModeWindSpeed::Auto)
        );
        assert_eq!(
            status.wind.vertical_direction.get_enum(),
            Some(VerticalDirection::Auto)
        );
        assert_eq!(
            status.wind.horizontal_direction.get_enum(),
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
        status.temperature.cooling.set_value(24.5);
        status.temperature.heating.set_value(25.0);
        status.temperature.automatic.set_value(0.0);
        status
            .wind
            .automode_speed
            .set_value(AutoModeWindSpeed::Silent);
        status.wind.speed.set_value(WindSpeed::Lev4);
        status
            .wind
            .vertical_direction
            .set_value(VerticalDirection::BottomMost);
        status
            .wind
            .horizontal_direction
            .set_value(HorizontalDirection::RightCenter);

        let req: DaikinRequest = status.into();
        let json = serde_json::to_value(req).unwrap();
        assert_eq!(
            json,
            serde_json::from_str::<serde_json::Value>(include_str!("./fixtures/update.json"))
                .unwrap()
        );
    }
}
