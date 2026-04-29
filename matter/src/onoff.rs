use dsiot::{PowerState, StateTransition};
use rs_matter::dm::clusters::decl::on_off;
use rs_matter::dm::{Cluster, Dataver, InvokeContext, ReadContext};
use rs_matter::error::{Error, ErrorCode};
use rs_matter::with;

use crate::device::Device;

pub struct OnOffHandler {
    pub(crate) dataver: Dataver,
    device: Device,
}

impl OnOffHandler {
    pub const CLUSTER: Cluster<'static> = on_off::FULL_CLUSTER
        .with_revision(6)
        .with_features(0)
        .with_attrs(with!(required; on_off::AttributeId::OnOff))
        .with_cmds(with!(
            on_off::CommandId::Off | on_off::CommandId::On | on_off::CommandId::Toggle
        ));

    pub fn new(dataver: Dataver, device: Device) -> Self {
        Self { dataver, device }
    }

    fn set_power(&self, power: PowerState) -> Result<(), Error> {
        let mut status = self.device.get_status().map_err(|e| {
            warn!("Failed to get status: {e}");
            Error::from(ErrorCode::Busy)
        })?;
        StateTransition::new()
            .power(power)
            .apply_to_status(&mut status)
            .map_err(|e| {
                warn!("State transition failed: {e}");
                Error::from(ErrorCode::InvalidState)
            })?;
        self.device.update(status).map_err(|e| {
            warn!("Failed to update: {e}");
            Error::from(ErrorCode::Busy)
        })?;
        self.dataver.changed();
        Ok(())
    }
}

impl on_off::ClusterHandler for OnOffHandler {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn on_off(&self, _ctx: impl ReadContext) -> Result<bool, Error> {
        let status = self.device.get_status().map_err(|e| {
            warn!("Failed to get status: {e}");
            Error::from(ErrorCode::Busy)
        })?;
        Ok(PowerState::from_status(&status) == Some(PowerState::On))
    }

    fn handle_off(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        debug!("OnOff: off");
        self.set_power(PowerState::Off)
    }

    fn handle_on(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        debug!("OnOff: on");
        self.set_power(PowerState::On)
    }

    fn handle_toggle(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        let status = self.device.get_status().map_err(|e| {
            warn!("Failed to get status: {e}");
            Error::from(ErrorCode::Busy)
        })?;
        let current = PowerState::from_status(&status).unwrap_or(PowerState::Off);
        let next = match current {
            PowerState::On => PowerState::Off,
            PowerState::Off => PowerState::On,
        };
        debug!("OnOff: toggle → {:?}", next);
        self.set_power(next)
    }

    fn handle_off_with_effect(
        &self,
        _ctx: impl InvokeContext,
        _req: on_off::OffWithEffectRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_on_with_recall_global_scene(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_on_with_timed_off(
        &self,
        _ctx: impl InvokeContext,
        _req: on_off::OnWithTimedOffRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }
}
