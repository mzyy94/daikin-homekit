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

pub async fn set_initial_value(
    status: DaikinStatus,
    service: &mut HeaterCoolerService,
) -> Result<(), Error> {
    service.active.set_value(status.power.into()).await?;
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
