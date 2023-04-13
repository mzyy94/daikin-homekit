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
        .cooling_threshold_temperature
        .as_mut()
        .unwrap()
        .set_value(status.target_cooling_temperature.into())
        .await?;
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
            println!("active characteristic read (async)");
            let status = dk.get_status().await.unwrap();
            Ok(status.power)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: u8, new_val: u8| {
        let dk = dk.clone();
        async move {
            println!("active updated from {} to {} (async)", current_val, new_val);
            if current_val == new_val {
                println!("- skip");
                return Ok(());
            }
            let mut status = dk.get_status().await.unwrap();
            status.power = Some(new_val);
            dk.update(status).await.unwrap();
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
            println!("current_heater_cooler_state characteristic read (async)");
            let status = dk.get_status().await.unwrap();
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

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: u8, new_val: u8| {
        let dk = dk.clone();
        async move {
            println!(
                "current_heater_cooler_state updated from {} to {} (async)",
                current_val, new_val
            );
            let _ = dk.get_status().await.unwrap();
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
            println!("target_heater_cooler_state characteristic read (async)");
            let status = dk.get_status().await.unwrap();
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
    char.on_update_async(Some(move |current_val: u8, new_val: u8| {
        let dk = dk.clone();
        async move {
            println!(
                "target_heater_cooler_state updated from {} to {} (async)",
                current_val, new_val
            );
            if current_val == new_val {
                println!("- skip");
                return Ok(());
            }
            let mut status = dk.get_status().await.unwrap();
            if let Some(mode) = match new_val {
                0 => Some(Mode::Auto),
                1 => Some(Mode::Heating),
                2 => Some(Mode::Cooling),
                _ => None,
            } {
                status.mode = Some(mode);
                dk.update(status).await.unwrap();
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
            println!("current_temperature characteristic read (async)");
            let status = dk.get_status().await.unwrap();
            Ok(status.current_temperature)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            println!(
                "current_temperature updated from {} to {} (async)",
                current_val, new_val
            );
            let _ = dk.get_status().await.unwrap();
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
            println!("heating_threshold_temperature characteristic read (async)");
            let status = dk.get_status().await.unwrap();
            Ok(status.target_heating_temperature)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            println!(
                "heating_threshold_temperature updated from {} to {} (async)",
                current_val, new_val
            );
            if current_val == new_val {
                println!("- skip");
                return Ok(());
            }
            let mut status = dk.get_status().await.unwrap();
            status.target_heating_temperature = Some(new_val);
            dk.update(status).await.unwrap();
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
            println!("cooling_threshold_temperature characteristic read (async)");
            let status = dk.get_status().await.unwrap();
            Ok(status.target_cooling_temperature)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            println!(
                "cooling_threshold_temperature updated from {} to {} (async)",
                current_val, new_val
            );
            if current_val == new_val {
                println!("- skip");
                return Ok(());
            }
            let mut status = dk.get_status().await.unwrap();
            status.target_cooling_temperature = Some(new_val);
            dk.update(status).await.unwrap();
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
            println!("rotation_speed characteristic read (async)");
            let status = dk.get_status().await.unwrap();
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
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            println!(
                "rotation_speed updated from {} to {} (async)",
                current_val, new_val
            );
            if current_val == new_val {
                println!("- skip");
                return Ok(());
            }
            let mut status = dk.get_status().await.unwrap();
            let speed = match new_val as u8 {
                0..=5 => WindSpeed::Silent,
                6..=20 => WindSpeed::Lev1,
                21..=35 => WindSpeed::Lev2,
                36..=50 => WindSpeed::Lev3,
                51..=75 => WindSpeed::Lev4,
                76..=90 => WindSpeed::Lev5,
                _ => WindSpeed::Auto,
            };
            let auto_speed = if new_val < 50.0 {
                AutoModeWindSpeed::Silent
            } else {
                AutoModeWindSpeed::Auto
            };
            status.wind_speed = Some(speed);
            status.automode_wind_speed = Some(auto_speed);
            dk.update(status).await.unwrap();
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
            println!("swing_mode characteristic read (async)");
            let status = dk.get_status().await.unwrap();
            let mode = match status.vertical_wind_direction {
                Some(VerticalDirection::Swing) => Some(1),
                _ => Some(0),
            };
            Ok(mode)
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: u8, new_val: u8| {
        let dk = dk.clone();
        async move {
            println!(
                "swing_mode updated from {} to {} (async)",
                current_val, new_val
            );
            if current_val == new_val {
                println!("- skip");
                return Ok(());
            }
            let mut status = dk.get_status().await.unwrap();
            if new_val == 0 {
                status.vertical_wind_direction = Some(VerticalDirection::Auto);
                status.horizontal_wind_direction = Some(HorizontalDirection::Auto);
            } else {
                status.vertical_wind_direction = Some(VerticalDirection::Swing);
                status.horizontal_wind_direction = Some(HorizontalDirection::Swing);
            }
            dk.update(status).await.unwrap();
            Ok(())
        }
        .boxed()
    }));
}
