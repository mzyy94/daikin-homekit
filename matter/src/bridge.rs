use rs_matter::dm::clusters::decl::bridged_device_basic_information;
use rs_matter::dm::clusters::decl::fan_control as rs_fan_control;
use rs_matter::dm::clusters::decl::thermostat as rs_thermostat;
use rs_matter::dm::clusters::decl::{identify, on_off};
use rs_matter::dm::clusters::desc::{self, ClusterHandler as _};
use rs_matter::dm::devices::{DEV_TYPE_AGGREGATOR, DEV_TYPE_BRIDGED_NODE};
use rs_matter::dm::{
    Dataver, DeviceType, Endpoint, Handler, InvokeContext, InvokeReply, Matcher, Node,
    NonBlockingHandler, OperationContext, ReadContext, ReadReply, WriteContext,
};
use rs_matter::error::{Error, ErrorCode};
use rs_matter::{clusters, devices, root_endpoint};

use crate::bridged_info::BridgedInfo;
use crate::identify::StubIdentify;
use crate::{device, fan_control, onoff, thermostat};

pub(crate) const DEV_TYPE_ROOM_AC: DeviceType = DeviceType {
    dtype: 0x0072,
    drev: 2,
};

const ROOT_EP: Endpoint<'static> = root_endpoint!(geth);

const AGGREGATOR_EP: Endpoint<'static> = Endpoint {
    id: 1,
    device_types: devices!(DEV_TYPE_AGGREGATOR),
    clusters: clusters!(desc::DescHandler::CLUSTER),
};

const BRIDGED_EP: Endpoint<'static> = Endpoint {
    id: 0, // placeholder, overridden at runtime
    device_types: devices!(DEV_TYPE_ROOM_AC, DEV_TYPE_BRIDGED_NODE),
    clusters: clusters!(
        desc::DescHandler::CLUSTER,
        StubIdentify::CLUSTER,
        BridgedInfo::CLUSTER,
        onoff::OnOffHandler::CLUSTER,
        thermostat::ThermostatHandler::CLUSTER,
        fan_control::FanControlHandler::CLUSTER
    ),
};

pub(crate) fn build_node(ep_ids: &[u16]) -> Node<'static> {
    let mut endpoints = vec![ROOT_EP, AGGREGATOR_EP];
    for &id in ep_ids {
        endpoints.push(Endpoint { id, ..BRIDGED_EP });
    }
    Node {
        endpoints: Box::leak(endpoints.into_boxed_slice()),
    }
}

pub(crate) struct BridgedDevice {
    pub(crate) ep_id: u16,
    desc: desc::HandlerAdaptor<desc::DescHandler<'static>>,
    identify: StubIdentify,
    bridged_info: BridgedInfo,
    pub(crate) on_off: onoff::OnOffHandler,
    pub(crate) therm: thermostat::ThermostatHandler,
    pub(crate) fan_ctl: fan_control::FanControlHandler,
    pub(crate) device: device::Device,
}

impl BridgedDevice {
    pub(crate) fn new(
        ep_id: u16,
        rand: &mut impl rand::RngCore,
        bridged_info: BridgedInfo,
        device: device::Device,
    ) -> Self {
        Self {
            ep_id,
            desc: desc::DescHandler::new(Dataver::new_rand(rand)).adapt(),
            identify: StubIdentify::new(Dataver::new_rand(rand)),
            bridged_info,
            on_off: onoff::OnOffHandler::new(Dataver::new_rand(rand), device.clone()),
            therm: thermostat::ThermostatHandler::new(Dataver::new_rand(rand), device.clone()),
            fan_ctl: fan_control::FanControlHandler::new(Dataver::new_rand(rand), device.clone()),
            device,
        }
    }
}

pub(crate) struct BridgeHandler {
    pub(crate) devices: Vec<BridgedDevice>,
}

impl BridgeHandler {
    fn find(&self, ep_id: u16) -> Option<&BridgedDevice> {
        self.devices.iter().find(|d| d.ep_id == ep_id)
    }
}

/// Matches any bridged endpoint (id >= 2).
pub(crate) struct BridgedMatcher;

impl Matcher for BridgedMatcher {
    fn matches(&self, ctx: impl OperationContext) -> bool {
        ctx.endpt() >= 2
    }
}

impl Handler for BridgeHandler {
    fn read(&self, ctx: impl ReadContext, reply: impl ReadReply) -> Result<(), Error> {
        let ep = ctx.endpt();
        let cl = ctx.cluster();
        let dev = self
            .find(ep)
            .ok_or(Error::from(ErrorCode::EndpointNotFound))?;

        if cl == desc::DescHandler::CLUSTER.id {
            dev.desc.read(ctx, reply)
        } else if cl == StubIdentify::CLUSTER.id {
            identify::HandlerAdaptor(&dev.identify).read(ctx, reply)
        } else if cl == BridgedInfo::CLUSTER.id {
            bridged_device_basic_information::HandlerAdaptor(&dev.bridged_info).read(ctx, reply)
        } else if cl == onoff::OnOffHandler::CLUSTER.id {
            on_off::HandlerAdaptor(&dev.on_off).read(ctx, reply)
        } else if cl == thermostat::ThermostatHandler::CLUSTER.id {
            rs_thermostat::HandlerAdaptor(&dev.therm).read(ctx, reply)
        } else if cl == fan_control::FanControlHandler::CLUSTER.id {
            rs_fan_control::HandlerAdaptor(&dev.fan_ctl).read(ctx, reply)
        } else {
            Err(ErrorCode::ClusterNotFound.into())
        }
    }

    fn write(&self, ctx: impl WriteContext) -> Result<(), Error> {
        let ep = ctx.endpt();
        let cl = ctx.cluster();
        let dev = self
            .find(ep)
            .ok_or(Error::from(ErrorCode::EndpointNotFound))?;

        if cl == StubIdentify::CLUSTER.id {
            identify::HandlerAdaptor(&dev.identify).write(ctx)
        } else if cl == onoff::OnOffHandler::CLUSTER.id {
            on_off::HandlerAdaptor(&dev.on_off).write(ctx)
        } else if cl == thermostat::ThermostatHandler::CLUSTER.id {
            rs_thermostat::HandlerAdaptor(&dev.therm).write(ctx)
        } else if cl == fan_control::FanControlHandler::CLUSTER.id {
            rs_fan_control::HandlerAdaptor(&dev.fan_ctl).write(ctx)
        } else {
            Err(ErrorCode::AttributeNotFound.into())
        }
    }

    fn invoke(&self, ctx: impl InvokeContext, reply: impl InvokeReply) -> Result<(), Error> {
        let ep = ctx.endpt();
        let cl = ctx.cluster();
        let dev = self
            .find(ep)
            .ok_or(Error::from(ErrorCode::EndpointNotFound))?;

        if cl == StubIdentify::CLUSTER.id {
            identify::HandlerAdaptor(&dev.identify).invoke(ctx, reply)
        } else if cl == onoff::OnOffHandler::CLUSTER.id {
            on_off::HandlerAdaptor(&dev.on_off).invoke(ctx, reply)
        } else if cl == thermostat::ThermostatHandler::CLUSTER.id {
            rs_thermostat::HandlerAdaptor(&dev.therm).invoke(ctx, reply)
        } else if cl == fan_control::FanControlHandler::CLUSTER.id {
            rs_fan_control::HandlerAdaptor(&dev.fan_ctl).invoke(ctx, reply)
        } else {
            Err(ErrorCode::CommandNotFound.into())
        }
    }
}

impl NonBlockingHandler for BridgeHandler {}
