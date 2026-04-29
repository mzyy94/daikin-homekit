use dsiot::{DaikinStatus, Mode, PowerState, StateTransition, TemperatureTarget};
use rs_matter::dm::clusters::decl::thermostat;
use rs_matter::dm::{Cluster, Dataver, InvokeContext, ReadContext, WriteContext};
use rs_matter::error::{Error, ErrorCode};
use rs_matter::tlv::{Nullable, TLVBuilderParent};
use rs_matter::with;

use crate::device::Device;

pub struct ThermostatHandler {
    pub(crate) dataver: Dataver,
    device: Device,
}

impl ThermostatHandler {
    pub const CLUSTER: Cluster<'static> = thermostat::FULL_CLUSTER
        .with_revision(7)
        .with_features(
            thermostat::Feature::HEATING.bits()
                | thermostat::Feature::COOLING.bits()
                | thermostat::Feature::AUTO_MODE.bits(),
        )
        .with_attrs(with!(
            required;
            thermostat::AttributeId::LocalTemperature
            | thermostat::AttributeId::OutdoorTemperature
            | thermostat::AttributeId::SystemMode
            | thermostat::AttributeId::OccupiedCoolingSetpoint
            | thermostat::AttributeId::OccupiedHeatingSetpoint
            | thermostat::AttributeId::ControlSequenceOfOperation
        ))
        .with_cmds(with!());

    pub fn new(dataver: Dataver, device: Device) -> Self {
        Self { dataver, device }
    }

    fn get_status(&self) -> Result<DaikinStatus, Error> {
        self.device.get_status().map_err(|e| {
            warn!("Failed to get status: {e}");
            Error::from(ErrorCode::Busy)
        })
    }

    fn update(&self, status: DaikinStatus) -> Result<(), Error> {
        self.device.update(status).map_err(|e| {
            warn!("Failed to update: {e}");
            Error::from(ErrorCode::Busy)
        })
    }
}

fn system_mode_from_status(status: &DaikinStatus) -> thermostat::SystemModeEnum {
    match PowerState::from_status(status) {
        Some(PowerState::Off) | None => thermostat::SystemModeEnum::Off,
        Some(PowerState::On) => match status.mode.get_enum() {
            Some(Mode::Auto) => thermostat::SystemModeEnum::Auto,
            Some(Mode::Cooling) => thermostat::SystemModeEnum::Cool,
            Some(Mode::Heating) => thermostat::SystemModeEnum::Heat,
            Some(Mode::Fan) => thermostat::SystemModeEnum::FanOnly,
            Some(Mode::Dehumidify) => thermostat::SystemModeEnum::Dry,
            _ => thermostat::SystemModeEnum::Off,
        },
    }
}

/// Convert dsiot f32 °C to Matter 0.01°C i16.
fn temp_to_matter(celsius: f32) -> i16 {
    (celsius * 100.0) as i16
}

/// Convert Matter 0.01°C i16 to dsiot f32 °C.
fn temp_from_matter(value: i16) -> f32 {
    value as f32 / 100.0
}

impl thermostat::ClusterHandler for ThermostatHandler {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn local_temperature(&self, _ctx: impl ReadContext) -> Result<Nullable<i16>, Error> {
        let status = self.get_status()?;
        match status.sensors.temperature.get_f32() {
            Some(t) => Ok(Nullable::some(temp_to_matter(t))),
            None => Ok(Nullable::none()),
        }
    }

    fn outdoor_temperature(&self, _ctx: impl ReadContext) -> Result<Nullable<i16>, Error> {
        let status = self.get_status()?;
        match status.sensors.outdoor_temperature.get_f32() {
            Some(t) => Ok(Nullable::some(temp_to_matter(t))),
            None => Ok(Nullable::none()),
        }
    }

    fn system_mode(&self, _ctx: impl ReadContext) -> Result<thermostat::SystemModeEnum, Error> {
        let status = self.get_status()?;
        Ok(system_mode_from_status(&status))
    }

    fn set_system_mode(
        &self,
        _ctx: impl WriteContext,
        value: thermostat::SystemModeEnum,
    ) -> Result<(), Error> {
        let mut status = self.get_status()?;
        let transition = match value {
            thermostat::SystemModeEnum::Off => StateTransition::new().turn_off(),
            thermostat::SystemModeEnum::Auto => StateTransition::new().turn_on().mode(Mode::Auto),
            thermostat::SystemModeEnum::Cool => {
                StateTransition::new().turn_on().mode(Mode::Cooling)
            }
            thermostat::SystemModeEnum::Heat => {
                StateTransition::new().turn_on().mode(Mode::Heating)
            }
            thermostat::SystemModeEnum::FanOnly => StateTransition::new().turn_on().mode(Mode::Fan),
            thermostat::SystemModeEnum::Dry => {
                StateTransition::new().turn_on().mode(Mode::Dehumidify)
            }
            _ => return Err(ErrorCode::ConstraintError.into()),
        };
        transition.apply_to_status(&mut status).map_err(|e| {
            warn!("State transition failed: {e}");
            Error::from(ErrorCode::InvalidState)
        })?;
        debug!("Thermostat: system_mode → {:?}", value);
        self.update(status)?;
        self.dataver.changed();
        Ok(())
    }

    fn occupied_cooling_setpoint(&self, _ctx: impl ReadContext) -> Result<i16, Error> {
        let status = self.get_status()?;
        match status.temperature.cooling.get_f32() {
            Some(t) => Ok(temp_to_matter(t)),
            None => Ok(2600), // 26.0°C
        }
    }

    fn set_occupied_cooling_setpoint(
        &self,
        _ctx: impl WriteContext,
        value: i16,
    ) -> Result<(), Error> {
        let mut status = self.get_status()?;
        let temp = temp_from_matter(value);
        TemperatureTarget::cooling(temp).apply_to_status(&mut status);
        debug!("Thermostat: cooling setpoint → {temp}°C");
        self.update(status)?;
        self.dataver.changed();
        Ok(())
    }

    fn occupied_heating_setpoint(&self, _ctx: impl ReadContext) -> Result<i16, Error> {
        let status = self.get_status()?;
        match status.temperature.heating.get_f32() {
            Some(t) => Ok(temp_to_matter(t)),
            None => Ok(2000), // 20.0°C
        }
    }

    fn set_occupied_heating_setpoint(
        &self,
        _ctx: impl WriteContext,
        value: i16,
    ) -> Result<(), Error> {
        let mut status = self.get_status()?;
        let temp = temp_from_matter(value);
        TemperatureTarget::heating(temp).apply_to_status(&mut status);
        debug!("Thermostat: heating setpoint → {temp}°C");
        self.update(status)?;
        self.dataver.changed();
        Ok(())
    }

    fn control_sequence_of_operation(
        &self,
        _ctx: impl ReadContext,
    ) -> Result<thermostat::ControlSequenceOfOperationEnum, Error> {
        Ok(thermostat::ControlSequenceOfOperationEnum::CoolingAndHeating)
    }

    fn set_control_sequence_of_operation(
        &self,
        _ctx: impl WriteContext,
        _value: thermostat::ControlSequenceOfOperationEnum,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidAction.into())
    }

    fn handle_setpoint_raise_lower(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::SetpointRaiseLowerRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_set_weekly_schedule(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::SetWeeklyScheduleRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_get_weekly_schedule<P: TLVBuilderParent>(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::GetWeeklyScheduleRequest<'_>,
        _response: thermostat::GetWeeklyScheduleResponseBuilder<P>,
    ) -> Result<P, Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_clear_weekly_schedule(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_set_active_schedule_request(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::SetActiveScheduleRequestRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_set_active_preset_request(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::SetActivePresetRequestRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_add_thermostat_suggestion<P: TLVBuilderParent>(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::AddThermostatSuggestionRequest<'_>,
        _response: thermostat::AddThermostatSuggestionResponseBuilder<P>,
    ) -> Result<P, Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_remove_thermostat_suggestion(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::RemoveThermostatSuggestionRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_atomic_request<P: TLVBuilderParent>(
        &self,
        _ctx: impl InvokeContext,
        _req: thermostat::AtomicRequestRequest<'_>,
        _response: thermostat::AtomicResponseBuilder<P>,
    ) -> Result<P, Error> {
        Err(ErrorCode::InvalidCommand.into())
    }
}
