use crate::status::DaikinStatus;
use crate::{daikin::Daikin, error::Error, status::Mode};
use futures::prelude::*;
use hap::characteristic::{
    active::ActiveCharacteristic,
    cooling_threshold_temperature::CoolingThresholdTemperatureCharacteristic,
    current_heater_cooler_state::CurrentHeaterCoolerStateCharacteristic,
    current_temperature::CurrentTemperatureCharacteristic,
    heating_threshold_temperature::HeatingThresholdTemperatureCharacteristic,
    target_heater_cooler_state::TargetHeaterCoolerStateCharacteristic,
    AsyncCharacteristicCallbacks, HapCharacteristic,
};
use hap::service::heater_cooler::HeaterCoolerService;
use serde_json::{Number, Value};

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
}

macro_rules! json_number {
    ($vopt:expr) => {
        Value::Number(Number::from_f64($vopt.unwrap_or_default()).unwrap())
    };
}

pub async fn set_initial_value(
    status: DaikinStatus,
    service: &mut HeaterCoolerService,
) -> Result<(), Error> {
    service
        .active
        .set_value(Value::Number(Number::from(
            if status.power().unwrap_or_default() {
                1
            } else {
                0
            },
        )))
        .await?;
    service
        .current_heater_cooler_state
        .set_value(Value::Number(Number::from(0)))
        .await?;
    service
        .target_heater_cooler_state
        .set_value(Value::Number(Number::from(0)))
        .await?;
    service
        .current_temperature
        .set_value(json_number!(status.current_temperature()))
        .await?;

    service
        .heating_threshold_temperature
        .as_mut()
        .unwrap()
        .set_value(json_number!(status.target_heating_temperature()))
        .await?;
    service
        .cooling_threshold_temperature
        .as_mut()
        .unwrap()
        .set_value(json_number!(status.target_cooling_temperature()))
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
            Ok(Some(if status.power().unwrap_or_default() {
                1
            } else {
                0
            }))
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: u8, new_val: u8| {
        let dk = dk.clone();
        async move {
            println!("active updated from {} to {} (async)", current_val, new_val);
            let mut status = dk.get_status().await.unwrap();
            status.set_power(new_val == 1).unwrap();
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
            match status.mode() {
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
            match status.mode() {
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
            let mut status = dk.get_status().await.unwrap();
            if let Some(mode) = match new_val {
                0 => Some(Mode::Auto),
                1 => Some(Mode::Heating),
                2 => Some(Mode::Cooling),
                _ => None,
            } {
                status.set_mode(mode).unwrap();
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
            Ok(status.current_temperature().map(|t| t as f32))
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            let _ = dk.get_status().await.unwrap();
            println!(
                "current_temperature updated from {} to {} (async)",
                current_val, new_val
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
            println!("heating_threshold_temperature characteristic read (async)");
            let status = dk.get_status().await.unwrap();
            Ok(status.target_heating_temperature().map(|t| t as f32))
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            let mut status = dk.get_status().await.unwrap();
            println!(
                "heating_threshold_temperature updated from {} to {} (async)",
                current_val, new_val
            );
            status
                .set_target_heating_temperature(f64::from(new_val))
                .unwrap();
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
            Ok(status.target_cooling_temperature().map(|t| t as f32))
        }
        .boxed()
    }));

    let dk = daikin.clone();
    char.on_update_async(Some(move |current_val: f32, new_val: f32| {
        let dk = dk.clone();
        async move {
            let mut status = dk.get_status().await.unwrap();
            println!(
                "cooling_threshold_temperature updated from {} to {} (async)",
                current_val, new_val
            );
            status
                .set_target_cooling_temperature(f64::from(new_val))
                .unwrap();
            dk.update(status).await.unwrap();
            Ok(())
        }
        .boxed()
    }));
}
