use super::property::{Item, Property};
use super::request::{DaikinRequest, Request};
use super::response::DaikinResponse;
use crate::types::{AutoModeWindSpeed, HorizontalDirection, Mode, VerticalDirection, WindSpeed};

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

impl From<DaikinStatus> for DaikinRequest {
    fn from(status: DaikinStatus) -> Self {
        let mut prop = Property::new_tree("dgc_status");

        set_child_prop!({ prop }.e_1002.e_A002.p_01 = status.power);
        set_child_prop!({ prop }.e_1002.e_3001.p_01 = status.mode);
        set_child_prop!({ prop }.e_1002.e_3001.p_02 = status.temperature.cooling);
        set_child_prop!({ prop }.e_1002.e_3001.p_03 = status.temperature.heating);
        set_child_prop!({ prop }.e_1002.e_3001.p_1F = status.temperature.automatic);
        set_child_prop!({ prop }.e_1002.e_3001.p_09 = status.wind.speed);
        set_child_prop!({ prop }.e_1002.e_3001.p_26 = status.wind.automode_speed);
        set_child_prop!({ prop }.e_1002.e_3001.p_05 = status.wind.vertical_direction);
        set_child_prop!({ prop }.e_1002.e_3001.p_06 = status.wind.horizontal_direction);

        DaikinRequest {
            requests: vec![Request {
                op: 3,
                pc: prop,
                to: "/dsiot/edge/adr_0100.dgc_status".into(),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter() {
        let res: DaikinResponse = serde_json::from_str(include_str!("../fixtures/status.json"))
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
        let res: DaikinResponse = serde_json::from_str(include_str!("../fixtures/status.json"))
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
            serde_json::from_str::<serde_json::Value>(include_str!("../fixtures/update.json"))
                .unwrap()
        );
    }
}
