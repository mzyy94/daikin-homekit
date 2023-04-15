use crate::status::{
    AutoModeWindSpeed, DaikinStatus, HorizontalDirection, VerticalDirection, WindSpeed,
};
use crate::{daikin::Daikin, error::Error, status::Mode};
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

pub fn setup_characteristic_callback(daikin: Daikin, service: &mut HeaterCoolerService) {
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
    setup_rotation_speed(daikin.clone(), service.rotation_speed.as_mut().unwrap());
    setup_swing_mode(daikin.clone(), service.swing_mode.as_mut().unwrap());
}

pub async fn set_initial_value(
    status: DaikinStatus,
    service: &mut HeaterCoolerService,
) -> Result<(), Error> {
    service.active.set_value(status.power.into()).await?;
    service
        .current_heater_cooler_state
        .set_value(
            match status.mode {
                Some(Mode::Fan) => 0,        // Inactive,
                Some(Mode::Dehumidify) => 1, // Idle
                Some(Mode::Heating) => 2,    // Heating
                Some(Mode::Cooling) => 3,    // Cooling
                _ => 0,
            }
            .into(),
        )
        .await?;
    service
        .target_heater_cooler_state
        .set_value(
            match status.mode {
                Some(Mode::Auto) => 0,    // Auto,
                Some(Mode::Heating) => 1, // Heat
                Some(Mode::Cooling) => 2, // Cool
                _ => 0,
            }
            .into(),
        )
        .await?;
    service
        .current_temperature
        .set_value(status.current_temperature.into())
        .await?;

    service
        .heating_threshold_temperature
        .as_mut()
        .unwrap()
        .set_value(status.target_heating_temperature.into())
        .await?;
    service
        .heating_threshold_temperature
        .as_mut()
        .unwrap()
        .set_step_value(Some(status.meta.target_heating_temperature.0 .0.into()))?;
    service
        .heating_threshold_temperature
        .as_mut()
        .unwrap()
        .set_min_value(
            status
                .meta
                .target_heating_temperature
                .0
                 .1
                .map(|v| v.into()),
        )?;
    service
        .heating_threshold_temperature
        .as_mut()
        .unwrap()
        .set_max_value(
            status
                .meta
                .target_heating_temperature
                .0
                 .2
                .map(|v| v.into()),
        )?;

    service
        .cooling_threshold_temperature
        .as_mut()
        .unwrap()
        .set_value(status.target_cooling_temperature.into())
        .await?;
    service
        .cooling_threshold_temperature
        .as_mut()
        .unwrap()
        .set_step_value(Some(status.meta.target_cooling_temperature.0 .0.into()))?;
    service
        .cooling_threshold_temperature
        .as_mut()
        .unwrap()
        .set_min_value(
            status
                .meta
                .target_cooling_temperature
                .0
                 .1
                .map(|v| v.into()),
        )?;
    service
        .cooling_threshold_temperature
        .as_mut()
        .unwrap()
        .set_max_value(
            status
                .meta
                .target_cooling_temperature
                .0
                 .2
                .map(|v| v.into()),
        )?;

    service
        .rotation_speed
        .as_mut()
        .unwrap()
        .set_value(
            match status.wind_speed {
                Some(WindSpeed::Silent) => Some(0.0),
                Some(WindSpeed::Lev1) => Some(10.0),
                Some(WindSpeed::Lev2) => Some(30.0),
                Some(WindSpeed::Lev3) => Some(50.0),
                Some(WindSpeed::Lev4) => Some(70.0),
                Some(WindSpeed::Lev5) => Some(90.0),
                Some(WindSpeed::Auto) => Some(100.0),
                _ => None,
            }
            .into(),
        )
        .await?;
    service
        .swing_mode
        .as_mut()
        .unwrap()
        .set_value(
            match status.vertical_wind_direction {
                Some(VerticalDirection::Swing) => Some(1),
                _ => Some(0),
            }
            .into(),
        )
        .await?;

    Ok(())
}

pub fn setup_active(daikin: Daikin, char: &mut ActiveCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("active read");
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            Ok(status.power)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            if cur == new {
                debug!("active updated from {} to {} - skip", cur, new);
                return Ok(());
            }
            debug!("active updated from {} to {}", cur, new);
            let mut status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(());
                }
            };
            status.power = Some(new);
            if let Err(e) = dk.update(status).await {
                error!("failed to call update {}", e);
            };
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_current_heater_cooler_state(
    daikin: Daikin,
    char: &mut CurrentHeaterCoolerStateCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("current_heater_cooler_state read");
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            match status.mode {
                Some(Mode::Fan) => Ok(Some(0)),        // Inactive
                Some(Mode::Dehumidify) => Ok(Some(1)), // Idle
                Some(Mode::Heating) => Ok(Some(2)),    // Heating
                Some(Mode::Cooling) => Ok(Some(3)),    // Cooling
                _ => Ok(None),
            }
        }
        .boxed()
    }));

    char.on_update_async(Some(move |cur: u8, new: u8| {
        async move {
            debug!(
                "current_heater_cooler_state updated from {} to {} - no action",
                cur, new
            );
            Ok(())
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
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            match status.mode {
                Some(Mode::Auto) => Ok(Some(0)),    // auto
                Some(Mode::Heating) => Ok(Some(1)), // heating
                Some(Mode::Cooling) => Ok(Some(2)), // cooling
                _ => Ok(None),
            }
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            if cur == new {
                debug!(
                    "target_heater_cooler_state updated from {} to {} - skip",
                    cur, new
                );
                return Ok(());
            }
            debug!("target_heater_cooler_state updated from {} to {}", cur, new);
            let mut status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(());
                }
            };
            if let Some(mode) = match new {
                0 => Some(Mode::Auto),
                1 => Some(Mode::Heating),
                2 => Some(Mode::Cooling),
                _ => None,
            } {
                status.mode = Some(mode);
                if let Err(e) = dk.update(status).await {
                    error!("failed to call update {}", e);
                };
            }

            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_current_temperature(daikin: Daikin, char: &mut CurrentTemperatureCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("current_temperature read");
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            Ok(status.current_temperature)
        }
        .boxed()
    }));

    char.on_update_async(Some(move |cur: f32, new: f32| {
        async move {
            debug!(
                "current_temperature updated from {} to {} - no action",
                cur, new
            );
            Ok(())
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
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            Ok(status.target_heating_temperature)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            if cur == new {
                debug!(
                    "heating_threshold_temperature updated from {} to {} - skip",
                    cur, new
                );
                return Ok(());
            }
            debug!(
                "heating_threshold_temperature updated from {} to {}",
                cur, new
            );
            let mut status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(());
                }
            };
            status.target_heating_temperature = Some(new);
            if let Err(e) = dk.update(status).await {
                error!("failed to call update {}", e);
            };
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
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            Ok(status.target_cooling_temperature)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            if cur == new {
                debug!(
                    "cooling_threshold_temperature updated from {} to {} - skip",
                    cur, new
                );
                return Ok(());
            }
            debug!(
                "cooling_threshold_temperature updated from {} to {}",
                cur, new
            );
            let mut status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(());
                }
            };
            status.target_cooling_temperature = Some(new);
            if let Err(e) = dk.update(status).await {
                error!("failed to call update {}", e);
            };
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
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            let speed = match status.wind_speed {
                Some(WindSpeed::Silent) => Some(5.0),
                Some(WindSpeed::Lev1) => Some(10.0),
                Some(WindSpeed::Lev2) => Some(30.0),
                Some(WindSpeed::Lev3) => Some(50.0),
                Some(WindSpeed::Lev4) => Some(70.0),
                Some(WindSpeed::Lev5) => Some(90.0),
                Some(WindSpeed::Auto) => Some(100.0),
                _ => None,
            };

            Ok(speed)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            if cur == new {
                debug!("rotation_speed updated from {} to {} - skip", cur, new);
                return Ok(());
            }
            debug!("rotation_speed updated from {} to {}", cur, new);
            let mut status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(());
                }
            };
            let speed = match new as u8 {
                0..=5 => WindSpeed::Silent,
                6..=20 => WindSpeed::Lev1,
                21..=35 => WindSpeed::Lev2,
                36..=50 => WindSpeed::Lev3,
                51..=75 => WindSpeed::Lev4,
                76..=90 => WindSpeed::Lev5,
                _ => WindSpeed::Auto,
            };
            let auto_speed = if new < 50.0 {
                AutoModeWindSpeed::Silent
            } else {
                AutoModeWindSpeed::Auto
            };
            status.wind_speed = Some(speed);
            status.automode_wind_speed = Some(auto_speed);
            if let Err(e) = dk.update(status).await {
                error!("failed to call update {}", e);
            };
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
            let status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(None);
                }
            };
            let mode = match status.vertical_wind_direction {
                Some(VerticalDirection::Swing) => Some(1),
                _ => Some(0),
            };
            Ok(mode)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            if cur == new {
                debug!("swing_mode updated from {} to {} - skip", cur, new);
                return Ok(());
            }
            debug!("swing_mode updated from {} to {}", cur, new);
            let mut status = match dk.get_status().await {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to call get_status {}", e);
                    return Ok(());
                }
            };
            if new == 0 {
                status.vertical_wind_direction = Some(VerticalDirection::Auto);
                status.horizontal_wind_direction = Some(HorizontalDirection::Auto);
            } else {
                status.vertical_wind_direction = Some(VerticalDirection::Swing);
                status.horizontal_wind_direction = Some(HorizontalDirection::Swing);
            }
            if let Err(e) = dk.update(status).await {
                error!("failed to call update {}", e);
            };
            Ok(())
        }
        .boxed()
    }));
}
