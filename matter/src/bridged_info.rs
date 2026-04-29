use dsiot::DaikinInfo;
use rs_matter::dm::clusters::decl::bridged_device_basic_information;
use rs_matter::dm::{Cluster, Dataver, InvokeContext, ReadContext};
use rs_matter::error::{Error, ErrorCode};
use rs_matter::tlv::{TLVBuilderParent, Utf8StrBuilder};
use rs_matter::with;

use crate::device::Device;

pub(crate) struct BridgedInfo {
    pub(crate) dataver: Dataver,
    device_name: &'static str,
    unique_id: &'static str,
    firmware_version: &'static str,
    device: Device,
}

impl BridgedInfo {
    pub(crate) const CLUSTER: Cluster<'static> = bridged_device_basic_information::FULL_CLUSTER
        .with_revision(4)
        .with_features(0)
        .with_attrs(with!(
            required;
            bridged_device_basic_information::AttributeId::VendorName
            | bridged_device_basic_information::AttributeId::ProductName
            | bridged_device_basic_information::AttributeId::NodeLabel
            | bridged_device_basic_information::AttributeId::SerialNumber
            | bridged_device_basic_information::AttributeId::SoftwareVersion
            | bridged_device_basic_information::AttributeId::SoftwareVersionString
            | bridged_device_basic_information::AttributeId::ProductURL
            | bridged_device_basic_information::AttributeId::Reachable
        ))
        .with_cmds(with!());

    pub(crate) fn new(dataver: Dataver, info: &DaikinInfo, device: Device) -> Self {
        Self {
            dataver,
            device_name: Box::leak(info.name.clone().into_boxed_str()),
            unique_id: Box::leak(info.mac.clone().into_boxed_str()),
            firmware_version: Box::leak(info.version.clone().into_boxed_str()),
            device,
        }
    }
}

impl bridged_device_basic_information::ClusterHandler for BridgedInfo {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn node_label<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set(self.device_name)
    }

    fn vendor_name<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set("Daikin")
    }

    fn product_name<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set("Air Conditioner")
    }

    fn serial_number<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set(self.unique_id)
    }

    fn software_version(&self, _ctx: impl ReadContext) -> Result<u32, Error> {
        Ok(1)
    }

    fn software_version_string<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set(self.firmware_version)
    }

    fn product_url<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set(env!("CARGO_PKG_REPOSITORY"))
    }

    fn reachable(&self, _ctx: impl ReadContext) -> Result<bool, Error> {
        Ok(self.device.is_reachable())
    }

    fn unique_id<P: TLVBuilderParent>(
        &self,
        _ctx: impl ReadContext,
        builder: Utf8StrBuilder<P>,
    ) -> Result<P, Error> {
        builder.set(self.unique_id)
    }

    fn handle_keep_active(
        &self,
        _ctx: impl InvokeContext,
        _req: bridged_device_basic_information::KeepActiveRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }
}
