use rs_matter::dm::{Cluster, Dataver, ReadContext};
use rs_matter::error::Error;
use rs_matter::tlv::Nullable;
use rs_matter::with;

use rs_matter::dm::clusters::decl::relative_humidity_measurement;

use crate::device::Device;

pub struct HumidityHandler {
    pub(crate) dataver: Dataver,
    device: Device,
}

impl HumidityHandler {
    pub const CLUSTER: Cluster<'static> = relative_humidity_measurement::FULL_CLUSTER
        .with_revision(3)
        .with_features(0)
        .with_attrs(with!(required))
        .with_cmds(with!());

    pub fn new(dataver: Dataver, device: Device) -> Self {
        Self { dataver, device }
    }
}

impl relative_humidity_measurement::ClusterHandler for HumidityHandler {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        let status = self.device.get_status().map_err(|e| {
            warn!("Failed to get status: {e}");
            Error::from(rs_matter::error::ErrorCode::Busy)
        })?;
        match status.sensors.humidity.get_f32() {
            Some(h) => Ok(Nullable::some((h * 100.0) as u16)),
            None => Ok(Nullable::none()),
        }
    }

    fn min_measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(0))
    }

    fn max_measured_value(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::some(10000))
    }
}
