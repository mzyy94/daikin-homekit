use crate::client::ReqwestClient;
use crate::fan_mapping;
use dsiot::daikin::Daikin;
use dsiot::mapping::{mode, swing};
use dsiot::property::{Binary, Metadata};
use dsiot::status::DaikinStatus;
use dsiot::{PowerState, StateTransition, TemperatureTarget, ValueConstraints};
use futures::prelude::*;
use hap::characteristic::{
    AsyncCharacteristicCallbacks, HapCharacteristic, active::ActiveCharacteristic,
    cooling_threshold_temperature::CoolingThresholdTemperatureCharacteristic,
    current_heater_cooler_state::CurrentHeaterCoolerStateCharacteristic,
    current_temperature::CurrentTemperatureCharacteristic,
    heating_threshold_temperature::HeatingThresholdTemperatureCharacteristic,
    rotation_speed::RotationSpeedCharacteristic, swing_mode::SwingModeCharacteristic,
    target_heater_cooler_state::TargetHeaterCoolerStateCharacteristic,
};
use hap::service::heater_cooler::HeaterCoolerService;
use serde_json::json;

pub async fn setup_characteristic(
    daikin: Daikin<ReqwestClient>,
    service: &mut HeaterCoolerService,
) -> anyhow::Result<()> {
    let status = daikin.get_status().await?;

    if status.wind_speed.get_enum().is_none()
        || matches!(
            status.clone().wind_speed.metadata,
            Metadata::Binary(Binary::Enum { max }) if max != "F80C"
        )
    {
        info!("wind_speed is not compatible. remove rotation_speed characteristic");
        service.rotation_speed = None;
    }

    if status.vertical_wind_direction.get_enum().is_none()
        || matches!( status.clone().vertical_wind_direction.metadata, Metadata::Binary(Binary::Enum { max }) if max != "3F808100")
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

fn setup_characteristic_callback(daikin: Daikin<ReqwestClient>, service: &mut HeaterCoolerService) {
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
    service
        .active
        .set_value(status.power.get_f32().map(|v| v as u8).into())
        .await?;
    service
        .current_heater_cooler_state
        .set_value(mode::to_current_state(status.mode.get_enum()).into())
        .await?;
    service
        .target_heater_cooler_state
        .set_value(mode::to_target_state(status.mode.get_enum()).into())
        .await?;
    service
        .current_temperature
        .set_value(status.current_temperature.get_f32().into())
        .await?;

    if let Some(char) = service.heating_threshold_temperature.as_mut() {
        char.set_value(status.target_heating_temperature.get_f32().into())
            .await?;
        if let Some(constraints) = ValueConstraints::from_item(&status.target_heating_temperature) {
            char.set_step_value(Some(constraints.step.into()))?;
            char.set_min_value(Some(constraints.min.into()))?;
            char.set_max_value(Some(constraints.max.into()))?;
        }
    }

    if let Some(char) = service.cooling_threshold_temperature.as_mut() {
        char.set_value(status.target_cooling_temperature.get_f32().into())
            .await?;
        if let Some(constraints) = ValueConstraints::from_item(&status.target_cooling_temperature) {
            char.set_step_value(Some(constraints.step.into()))?;
            char.set_min_value(Some(constraints.min.into()))?;
            char.set_max_value(Some(constraints.max.into()))?;
        }
    }

    if let Some(char) = service.rotation_speed.as_mut() {
        char.set_value(fan_mapping::speed_to_scale(status.wind_speed.get_enum()).into())
            .await?;
        let fan_constraints = fan_mapping::fan_speed_constraints();
        char.set_step_value(Some(json!(fan_constraints.step)))?;
        char.set_min_value(Some(json!(fan_constraints.min)))?;
        char.set_max_value(Some(json!(fan_constraints.max)))?;
    }

    if let Some(char) = service.swing_mode.as_mut() {
        char.set_value(
            Some(swing::to_enabled(status.vertical_wind_direction.get_enum()) as i32).into(),
        )
        .await?;
    }

    Ok(())
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

pub fn setup_active(daikin: Daikin<ReqwestClient>, char: &mut ActiveCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("active read");
            let status = dk.get_status().await?;
            let power = PowerState::from_status(&status);
            Ok(power.map(|p| if p == PowerState::On { 1u8 } else { 0u8 }))
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("active", cur, new);
            let mut status = dk.get_status().await?;
            let power = if new != 0 {
                PowerState::On
            } else {
                PowerState::Off
            };
            StateTransition::new()
                .power(power)
                .apply_to_status(&mut status)?;
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_current_heater_cooler_state(
    daikin: Daikin<ReqwestClient>,
    char: &mut CurrentHeaterCoolerStateCharacteristic,
) {
    let dk = daikin;
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("current_heater_cooler_state read");
            let status = dk.get_status().await?;
            Ok(Some(mode::to_current_state(status.mode.get_enum())))
        }
        .boxed()
    }));
}

pub fn setup_target_heater_cooler_state(
    daikin: Daikin<ReqwestClient>,
    char: &mut TargetHeaterCoolerStateCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("target_heater_cooler_state read");
            let status = dk.get_status().await?;
            Ok(mode::to_target_state(status.mode.get_enum()))
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("target_heater_cooler_state", cur, new);
            let mut status = dk.get_status().await?;
            if let Some(m) = mode::from_target_state(new) {
                StateTransition::new()
                    .mode(m)
                    .apply_to_status(&mut status)?;
                dk.update(status).await?;
            }

            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_current_temperature(
    daikin: Daikin<ReqwestClient>,
    char: &mut CurrentTemperatureCharacteristic,
) {
    let dk = daikin;
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("current_temperature read");
            let status = dk.get_status().await?;
            Ok(status.current_temperature.get_f32())
        }
        .boxed()
    }));
}

pub fn setup_heating_threshold_temperature(
    daikin: Daikin<ReqwestClient>,
    char: &mut HeatingThresholdTemperatureCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("heating_threshold_temperature read");
            let status = dk.get_status().await?;
            Ok(status.target_heating_temperature.get_f32())
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("heating_threshold_temperature", cur, new);
            let mut status = dk.get_status().await?;
            TemperatureTarget::heating(new).apply_to_status(&mut status);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_cooling_threshold_temperature(
    daikin: Daikin<ReqwestClient>,
    char: &mut CoolingThresholdTemperatureCharacteristic,
) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("cooling_threshold_temperature read");
            let status = dk.get_status().await?;
            Ok(status.target_cooling_temperature.get_f32())
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("cooling_threshold_temperature", cur, new);
            let mut status = dk.get_status().await?;
            TemperatureTarget::cooling(new).apply_to_status(&mut status);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_rotation_speed(daikin: Daikin<ReqwestClient>, char: &mut RotationSpeedCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("rotation_speed read");
            let status = dk.get_status().await?;
            Ok(fan_mapping::speed_to_scale(status.wind_speed.get_enum()))
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: f32, new: f32| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("rotation_speed", cur, new);
            let mut status = dk.get_status().await?;
            status.wind_speed.set_value(fan_mapping::scale_to_speed(new));
            status
                .automode_wind_speed
                .set_value(fan_mapping::scale_to_auto_mode(new));
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}

pub fn setup_swing_mode(daikin: Daikin<ReqwestClient>, char: &mut SwingModeCharacteristic) {
    let dk = daikin.clone();
    char.on_read_async(Some(move || {
        let dk = dk.clone();
        async move {
            debug!("swing_mode read");
            let status = dk.get_status().await?;
            let enabled = swing::to_enabled(status.vertical_wind_direction.get_enum());
            Ok(Some(enabled as u8))
        }
        .boxed()
    }));

    let dk = daikin;
    char.on_update_async(Some(move |cur: u8, new: u8| {
        let dk = dk.clone();
        async move {
            update_assert_ne!("swing_mode", cur, new);
            let mut status = dk.get_status().await?;
            let (vertical, horizontal) = swing::from_enabled(new != 0);
            status.vertical_wind_direction.set_value(vertical);
            status.horizontal_wind_direction.set_value(horizontal);
            dk.update(status).await?;
            Ok(())
        }
        .boxed()
    }));
}
