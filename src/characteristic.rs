use crate::status::{
    AutoModeWindSpeed, DaikinStatus, HorizontalDirection, VerticalDirection, WindSpeed,
};
use crate::{daikin::Daikin, status::Mode};
use futures::prelude::*;
use hap::characteristic::{
    active::ActiveCharacteristic,
    cooling_threshold_temperature::CoolingThresholdTemperatureCharacteristic,
    current_heater_cooler_state::CurrentHeaterCoolerStateCharacteristic,
    current_temperature::CurrentTemperatureCharacteristic,
    heating_threshold_temperature::HeatingThresholdTemperatureCharacteristic,
    rotation_speed::RotationSpeedCharacteristic, swing_mode::SwingModeCharacteristic,
    target_heater_cooler_state::TargetHeaterCoolerStateCharacteristic,
    AsyncCharacteristicCallbacks, HapCharacteristic,
};
use hap::service::heater_cooler::HeaterCoolerService;
use serde_json::json;

pub async fn setup_characteristic(
    daikin: Daikin,
    service: &mut HeaterCoolerService,
) -> anyhow::Result<()> {
    let status = daikin.get_status().await?;

    if status.wind_speed.is_none() || status.meta.wind_speed.max.map(|v| v as u32) != Some(0x0cf8) {
        info!("wind_speed is not compatible. remove rotation_speed characteristic");
        service.rotation_speed = None;
    }

    if status.vertical_wind_direction.is_none()
        || status.meta.vertical_wind_direction.max.map(|v| v as u32) != Some(0x0081803f)
    {
        info!("vertical_wind_direction is not compatible. remove swing_mode characteristic");
        service.swing_mode = None;
    }

    service.lock_physical_controls = None;
    service.name = None;
    service.temperature_display_units = None;

    setup_characteristic_callback(daikin, service);
    set_initial_value(status, service).await
}

fn setup_characteristic_callback(daikin: Daikin, service: &mut HeaterCoolerService) {
    setup_active(daikin.clone(), &mut service.active);
    setup_current_heater_cooler_state(daikin.clone(), &mut service.current_heater_cooler_state);
    setup_target_heater_cooler_state(daikin.clone(), &mut service.target_heater_cooler_state);
    setup_current_temperature(daikin.clone(), &mut service.current_temperature);
    setup_heating_threshold_temperature(
        daikin.clone(),
        service.heating_threshold_temperature.as_mut().unwrap(),
    );
    setup_cooling_threshold_temperature(
        daikin.clone(),
        service.cooling_threshold_temperature.as_mut().unwrap(),
    );
    if let Some(char) = service.rotation_speed.as_mut() {
        setup_rotation_speed(daikin.clone(), char);
    }
    if let Some(char) = service.swing_mode.as_mut() {
        setup_swing_mode(daikin, char);
    }
}

async fn set_initial_value(
    status: DaikinStatus,
    service: &mut HeaterCoolerService,
) -> anyhow::Result<()> {
    service.active.set_value(status.power.into()).await?;
    service
        .current_heater_cooler_state
        .set_value(current_mode_mapping(status.mode).into())
        .await?;
    service
        .target_heater_cooler_state
        .set_value(target_mode_mapping(status.mode).into())
        .await?;
    service
        .current_temperature
        .set_value(status.current_temperature.into())
        .await?;

    if let Some(char) = service.heating_threshold_temperature.as_mut() {
        char.set_value(status.target_heating_temperature.into())
            .await?;
        char.set_step_value(Some(status.meta.target_heating_temperature.step.into()))?;
        char.set_min_value(status.meta.target_heating_temperature.min.map(|v| v.into()))?;
        char.set_max_value(status.meta.target_heating_temperature.max.map(|v| v.into()))?;
    }

    if let Some(char) = service.cooling_threshold_temperature.as_mut() {
        char.set_value(status.target_cooling_temperature.into())
            .await?;
        char.set_step_value(Some(status.meta.target_cooling_temperature.step.into()))?;
        char.set_min_value(status.meta.target_cooling_temperature.min.map(|v| v.into()))?;
        char.set_max_value(status.meta.target_cooling_temperature.max.map(|v| v.into()))?;
    }

    if let Some(char) = service.rotation_speed.as_mut() {
        char.set_value(windspeed_mapping(status.wind_speed).into())
            .await?;
        char.set_step_value(Some(json!(1.0)))?;
        char.set_min_value(Some(json!(0.0)))?;
        char.set_max_value(Some(json!(7.0)))?;
    }

    if let Some(char) = service.swing_mode.as_mut() {
        char.set_value(
            Some((status.vertical_wind_direction == Some(VerticalDirection::Swing)) as i32).into(),
        )
        .await?;
    }

    Ok(())
}

fn windspeed_mapping(wind_speed: Option<WindSpeed>) -> Option<f32> {
    match wind_speed {
        Some(WindSpeed::Silent) => Some(1.0),
        Some(WindSpeed::Lev1) => Some(2.0),
        Some(WindSpeed::Lev2) => Some(3.0),
        Some(WindSpeed::Lev3) => Some(4.0),
        Some(WindSpeed::Lev4) => Some(5.0),
        Some(WindSpeed::Lev5) => Some(6.0),
        Some(WindSpeed::Auto) => Some(7.0),
        _ => None,
    }
}

fn current_mode_mapping(mode: Option<Mode>) -> Option<u8> {
    match mode {
        Some(Mode::Fan) => Some(0),        // Inactive,
        Some(Mode::Dehumidify) => Some(1), // Idle
        Some(Mode::Heating) => Some(2),    // Heating
        Some(Mode::Cooling) => Some(3),    // Cooling
        _ => None,
    }
}

fn target_mode_mapping(mode: Option<Mode>) -> Option<u8> {
    match mode {
        Some(Mode::Auto) => Some(0),    // Auto,
        Some(Mode::Heating) => Some(1), // Heat
        Some(Mode::Cooling) => Some(2), // Cool
        _ => None,
    }
}

macro_rules! update_assert_ne {
    ($name:expr, $cur:expr, $new:expr) => {
        if $cur == $new {
            debug!("{} updated from {} to {} - skip", $name, $cur, $new);
            return Ok(());
        }
        debug!("{} updated from {} to {}", $name, $cur, $new);
    };
}

pub fn setup_active(daikin: Daikin, char: &mut ActiveCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("active read");
            let status = dk.get_status().await?;
            Ok(status.power)
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("active", cur, new);
            let mut status = dk.get_status().await?;
            status.power = Some(new);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_current_heater_cooler_state(
    daikin: Daikin,
    char: &mut CurrentHeaterCoolerStateCharacteristic,
) {
    let dk = daikin;
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("current_heater_cooler_state read");
            let status = dk.get_status().await?;
            Ok(current_mode_mapping(status.mode))
        }
        .boxed()
    }));
}

pub fn setup_target_heater_cooler_state(
    daikin: Daikin,
    char: &mut TargetHeaterCoolerStateCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("target_heater_cooler_state read");
            let status = dk.get_status().await?;
            Ok(target_mode_mapping(status.mode))
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("target_heater_cooler_state", cur, new);
            let mut status = dk.get_status().await?;
            if let Some(mode) = match new {
                0 => Some(Mode::Auto),
                1 => Some(Mode::Heating),
                2 => Some(Mode::Cooling),
                _ => None,
            } {
                status.mode = Some(mode);
                dk.update(status).await?;
            }

            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_current_temperature(daikin: Daikin, char: &mut CurrentTemperatureCharacteristic) {
    let dk = daikin;
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("current_temperature read");
            let status = dk.get_status().await?;
            Ok(status.current_temperature)
        }
        .boxed()
    }));
}

pub fn setup_heating_threshold_temperature(
    daikin: Daikin,
    char: &mut HeatingThresholdTemperatureCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("heating_threshold_temperature read");
            let status = dk.get_status().await?;
            Ok(status.target_heating_temperature)
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("heating_threshold_temperature", cur, new);
            let mut status = dk.get_status().await?;
            status.target_heating_temperature = Some(new);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_cooling_threshold_temperature(
    daikin: Daikin,
    char: &mut CoolingThresholdTemperatureCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("cooling_threshold_temperature read");
            let status = dk.get_status().await?;
            Ok(status.target_cooling_temperature)
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("cooling_threshold_temperature", cur, new);
            let mut status = dk.get_status().await?;
            status.target_cooling_temperature = Some(new);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_rotation_speed(daikin: Daikin, char: &mut RotationSpeedCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("rotation_speed read");
            let status = dk.get_status().await?;
            let speed = windspeed_mapping(status.wind_speed);
            Ok(speed)
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("rotation_speed", cur, new);
            let mut status = dk.get_status().await?;
            let speed = match new as u8 {
                1 => WindSpeed::Silent,
                2 => WindSpeed::Lev1,
                3 => WindSpeed::Lev2,
                4 => WindSpeed::Lev3,
                5 => WindSpeed::Lev4,
                6 => WindSpeed::Lev5,
                _ => WindSpeed::Auto,
            };
            let auto_speed = if new < 50.0 {
                AutoModeWindSpeed::Silent
            } else {
                AutoModeWindSpeed::Auto
            };
            status.wind_speed = Some(speed);
            status.automode_wind_speed = Some(auto_speed);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_swing_mode(daikin: Daikin, char: &mut SwingModeCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("swing_mode read");
            let status = dk.get_status().await?;
            let mode = match status.vertical_wind_direction {
                Some(VerticalDirection::Swing) => Some(1),
                _ => Some(0),
            };
            Ok(mode)
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("swing_mode", cur, new);
            let mut status = dk.get_status().await?;
            if new == 0 {
                status.vertical_wind_direction = Some(VerticalDirection::Auto);
                status.horizontal_wind_direction = Some(HorizontalDirection::Auto);
            } else {
                status.vertical_wind_direction = Some(VerticalDirection::Swing);
                status.horizontal_wind_direction = Some(HorizontalDirection::Swing);
            }
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}
