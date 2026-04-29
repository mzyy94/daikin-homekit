use std::cell::Cell;
use std::time::Instant;

use dsiot::protocol::DaikinInfo;
use rs_matter::dm::clusters::decl::wi_fi_network_diagnostics;
use rs_matter::dm::{Cluster, Dataver, InvokeContext, ReadContext};
use rs_matter::error::Error;
use rs_matter::tlv::{Nullable, NullableBuilder, OctetsBuilder, TLVBuilderParent};
use rs_matter::with;

use wi_fi_network_diagnostics::SecurityTypeEnum;

use crate::device::Device;

const RSSI_CACHE_TTL_SECS: u64 = 60;

pub struct WifiDiagHandler {
    pub(crate) dataver: Dataver,
    info: DaikinInfo,
    device: Device,
    cached_rssi: Cell<Option<(Instant, i8)>>,
}

impl WifiDiagHandler {
    pub const CLUSTER: Cluster<'static> = wi_fi_network_diagnostics::FULL_CLUSTER
        .with_revision(1)
        .with_features(0)
        .with_attrs(with!(required))
        .with_cmds(with!());

    pub fn new(dataver: Dataver, info: DaikinInfo, device: Device) -> Self {
        Self {
            dataver,
            info,
            device,
            cached_rssi: Cell::new(None),
        }
    }

    fn get_rssi(&self) -> Option<i8> {
        if let Some((ts, rssi)) = self.cached_rssi.get()
            && ts.elapsed().as_secs() < RSSI_CACHE_TTL_SECS
        {
            return Some(rssi);
        }
        match self.device.get_info() {
            Ok(info) => {
                if let Some(rssi) = info.rssi {
                    self.cached_rssi.set(Some((Instant::now(), rssi)));
                }
                info.rssi
            }
            Err(e) => {
                warn!("Failed to get info for RSSI: {e}");
                self.cached_rssi.get().map(|(_, rssi)| rssi)
            }
        }
    }
}

impl wi_fi_network_diagnostics::ClusterHandler for WifiDiagHandler {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }

    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn bssid<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: NullableBuilder<P, OctetsBuilder<P>>,
    ) -> Result<P, Error> {
        builder.null()
    }

    fn security_type(&self, _ctx: impl ReadContext) -> Result<Nullable<SecurityTypeEnum>, Error> {
        let sec = self.info.security_type.as_deref().map(|s| match s {
            "WEP" => SecurityTypeEnum::WEP,
            "WPA" => SecurityTypeEnum::WPA,
            "WPA2" => SecurityTypeEnum::WPA2,
            "WPA3" => SecurityTypeEnum::WPA3,
            "NONE" => SecurityTypeEnum::None,
            _ => SecurityTypeEnum::Unspecified,
        });
        Ok(match sec {
            Some(v) => Nullable::some(v),
            None => Nullable::none(),
        })
    }

    fn wi_fi_version(
        &self,
        _ctx: impl ReadContext,
    ) -> Result<Nullable<wi_fi_network_diagnostics::WiFiVersionEnum>, Error> {
        Ok(Nullable::none())
    }

    fn channel_number(&self, _ctx: impl ReadContext) -> Result<Nullable<u16>, Error> {
        Ok(Nullable::none())
    }

    fn rssi(&self, _ctx: impl ReadContext) -> Result<Nullable<i8>, Error> {
        Ok(match self.get_rssi() {
            Some(v) => Nullable::some(v),
            None => Nullable::none(),
        })
    }

    fn handle_reset_counts(&self, _ctx: impl InvokeContext) -> Result<(), Error> {
        Ok(())
    }
}
