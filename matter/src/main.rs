#[macro_use]
extern crate log;

mod device;
mod fan_control;
mod onoff;
mod thermostat;

use core::pin::pin;
use std::net::{Ipv4Addr, UdpSocket};
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use daikin_client::{Daikin, ReqwestClient, discovery};
use dsiot::DaikinInfo;
use futures_lite::StreamExt;

use embassy_futures::select::{select, select4};
use static_cell::StaticCell;

use rs_matter::crypto::{Crypto, default_crypto};
use rs_matter::dm::clusters::basic_info::BasicInfoConfig;
use rs_matter::dm::clusters::decl::bridged_device_basic_information;
use rs_matter::dm::clusters::decl::fan_control as rs_fan_control;
use rs_matter::dm::clusters::decl::thermostat as rs_thermostat;
use rs_matter::dm::clusters::decl::{identify, on_off};
use rs_matter::dm::clusters::desc::{self, ClusterHandler as _};
use rs_matter::dm::clusters::dev_att::DeviceAttestation;
use rs_matter::dm::clusters::net_comm::SharedNetworks;
use rs_matter::dm::devices::test::{TEST_DEV_ATT, TEST_DEV_COMM};
use rs_matter::dm::devices::{DEV_TYPE_AGGREGATOR, DEV_TYPE_BRIDGED_NODE};
use rs_matter::dm::events::NoEvents;
use rs_matter::dm::networks::eth::EthNetwork;
use rs_matter::dm::networks::unix::UnixNetifs;
use rs_matter::dm::subscriptions::Subscriptions;
use rs_matter::dm::{
    Async, Cluster, DataModel, Dataver, DeviceType, EmptyHandler, Endpoint, EpClMatcher, Handler,
    IMBuffer, InvokeContext, InvokeReply, Matcher, Node, NonBlockingHandler, OperationContext,
    ReadContext, ReadReply, WriteContext, endpoints,
};
use rs_matter::error::{Error, ErrorCode};
use rs_matter::pairing::{DiscoveryCapabilities, qr::QrTextType};
use rs_matter::persist::{DirKvBlobStore, SharedKvBlobStore};
use rs_matter::respond::DefaultResponder;
use rs_matter::sc::pase::MAX_COMM_WINDOW_TIMEOUT_SECS;
use rs_matter::tlv::{TLVBuilderParent, Utf8StrBuilder};
use rs_matter::transport::MATTER_SOCKET_BIND_ADDR;
use rs_matter::utils::init::InitMaybeUninit;
use rs_matter::utils::select::Coalesce;
use rs_matter::utils::storage::pooled::PooledBuffers;
use rs_matter::{MATTER_PORT, Matter, clusters, devices, root_endpoint, with};

static MATTER: StaticCell<Matter> = StaticCell::new();
static BUFFERS: StaticCell<PooledBuffers<10, IMBuffer>> = StaticCell::new();
static SUBSCRIPTIONS: StaticCell<Subscriptions> = StaticCell::new();
static KV_BUF: StaticCell<[u8; 4096]> = StaticCell::new();

const DEV_TYPE_ROOM_AC: DeviceType = DeviceType {
    dtype: 0x0072,
    drev: 2,
};

const BRIDGE_DEV_DET: BasicInfoConfig<'static> = BasicInfoConfig {
    vid: 0xfff1,
    pid: 0x8001,
    product_name: "Daikin Matter Bridge",
    vendor_name: "daikin-matter",
    device_name: "Daikin Matter Bridge",
    hw_ver: 1,
    hw_ver_str: "1",
    sw_ver: 1,
    sw_ver_str: env!("CARGO_PKG_VERSION"),
    serial_no: "daikin-matter",
    product_label: "Daikin Matter Bridge",
    product_url: env!("CARGO_PKG_REPOSITORY"),
    ..BasicInfoConfig::new()
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

fn build_node(device_count: usize) -> Node<'static> {
    let mut endpoints = vec![ROOT_EP, AGGREGATOR_EP];
    for i in 0..device_count {
        endpoints.push(Endpoint {
            id: 2 + i as u16,
            ..BRIDGED_EP
        });
    }
    Node {
        endpoints: Box::leak(endpoints.into_boxed_slice()),
    }
}

struct StubIdentify {
    dataver: Dataver,
}

impl StubIdentify {
    const CLUSTER: Cluster<'static> = identify::FULL_CLUSTER;

    fn new(dataver: Dataver) -> Self {
        Self { dataver }
    }
}

impl identify::ClusterHandler for StubIdentify {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn identify_time(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        Ok(0)
    }

    fn identify_type(&self, _ctx: impl ReadContext) -> Result<identify::IdentifyTypeEnum, Error> {
        Ok(identify::IdentifyTypeEnum::None)
    }

    fn set_identify_time(&self, _ctx: impl WriteContext, _value: u16) -> Result<(), Error> {
        Ok(())
    }

    fn handle_identify(
        &self,
        _ctx: impl InvokeContext,
        _req: identify::IdentifyRequest<'_>,
    ) -> Result<(), Error> {
        info!("Identify requested");
        Ok(())
    }

    fn handle_trigger_effect(
        &self,
        _ctx: impl InvokeContext,
        _req: identify::TriggerEffectRequest<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

struct BridgedInfo {
    dataver: Dataver,
    device_name: &'static str,
    unique_id: &'static str,
}

impl BridgedInfo {
    const CLUSTER: Cluster<'static> = bridged_device_basic_information::FULL_CLUSTER
        .with_features(0)
        .with_attrs(with!(required))
        .with_cmds(with!());

    fn new(dataver: Dataver, info: &DaikinInfo) -> Self {
        Self {
            dataver,
            device_name: Box::leak(info.name.clone().into_boxed_str()),
            unique_id: Box::leak(info.mac.clone().into_boxed_str()),
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

    fn reachable(&self, _ctx: impl ReadContext) -> Result<bool, Error> {
        Ok(true)
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

struct BridgedDevice {
    ep_id: u16,
    desc: desc::HandlerAdaptor<desc::DescHandler<'static>>,
    identify: StubIdentify,
    bridged_info: BridgedInfo,
    pub(crate) on_off: onoff::OnOffHandler,
    pub(crate) therm: thermostat::ThermostatHandler,
    pub(crate) fan_ctl: fan_control::FanControlHandler,
    pub(crate) device: device::Device,
}

struct BridgeHandler {
    pub(crate) devices: Vec<BridgedDevice>,
}

impl BridgeHandler {
    fn find(&self, ep_id: u16) -> Option<&BridgedDevice> {
        self.devices.iter().find(|d| d.ep_id == ep_id)
    }
}

/// Matches any bridged endpoint (id >= 2).
struct BridgedMatcher;

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

fn dm_handler<'a>(
    mut rand: impl rand::RngCore,
    bridge: &'a BridgeHandler,
    node: Node<'static>,
) -> impl rs_matter::dm::AsyncMetadata + rs_matter::dm::AsyncHandler + 'a {
    let agg_desc_dataver = Dataver::new_rand(&mut rand);

    (
        node,
        endpoints::with_eth_sys(
            &false,
            &(),
            &UnixNetifs,
            rand,
            EmptyHandler
                .chain(
                    EpClMatcher::new(Some(1), Some(desc::DescHandler::CLUSTER.id)),
                    Async(desc::DescHandler::new_aggregator(agg_desc_dataver).adapt()),
                )
                .chain(BridgedMatcher, Async(bridge)),
        ),
    )
}

async fn run_mdns(matter: &Matter<'_>) -> Result<(), Error> {
    rs_matter::transport::network::mdns::astro::AstroMdnsResponder::new(matter)
        .run()
        .await
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// IPv4 address of Daikin AC
    #[arg(value_name = "ip_address")]
    ip_addrs: Vec<Ipv4Addr>,

    /// Discovery timeout in milliseconds
    #[arg(long, default_value = "3000")]
    timeout: u64,

    /// Expected number of devices to discover
    #[arg(long, default_value = "128", hide_default_value = true)]
    count: usize,

    /// Directory to store persistent data (pairing, fabrics, etc.)
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("daikin-matter")
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("daikin_matter=debug,rs_matter=info"),
    )
    .init();

    let cli = Cli::parse();

    // Connect to devices (specified or auto-discovered)
    let rt = tokio::runtime::Runtime::new()?;
    let connections: Vec<(Daikin<ReqwestClient>, DaikinInfo)> = rt.block_on(async {
        let mut conns = Vec::new();
        if cli.ip_addrs.is_empty() {
            info!("No IP addresses specified, discovering devices...");
            let timeout = Duration::from_millis(cli.timeout);
            let stream = discovery(timeout).await;
            let mut stream = pin!(stream);
            while let Some(result) = stream.next().await {
                match result {
                    Ok((dk, info)) => {
                        debug!("Status: {:?}", dk.get_status().await?);
                        conns.push((dk, info));
                        if conns.len() >= cli.count {
                            break;
                        }
                    }
                    Err(e) => warn!("Discovery error: {e}"),
                }
            }
        } else {
            for ip in &cli.ip_addrs {
                let dk = Daikin::new(*ip, ReqwestClient::try_new()?);
                let info = dk.get_info().await?;
                info!(
                    "Device: {} (MAC: {}, EDID: {})",
                    info.name, info.mac, info.edid
                );
                let status = dk.get_status().await?;
                debug!("Status: {:?}", status);
                conns.push((dk, info));
            }
        }
        if conns.is_empty() {
            anyhow::bail!("No devices found");
        }
        if cli.count != 128 && conns.len() < cli.count {
            anyhow::bail!(
                "Found only {} devices, but requested {}",
                conns.len(),
                cli.count
            );
        }
        anyhow::Ok(conns)
    })?;

    let rt_handle = rt.handle().clone();
    let data_dir = cli.data_dir.unwrap_or_else(default_data_dir);
    info!("Data directory: {}", data_dir.display());

    // Run Matter stack on a separate thread with increased stack
    let thread = std::thread::Builder::new()
        .stack_size(550 * 1024)
        .spawn(move || run_matter(connections, rt_handle, data_dir))
        .unwrap();

    thread.join().unwrap()
}

fn run_matter(
    connections: Vec<(Daikin<ReqwestClient>, DaikinInfo)>,
    rt_handle: tokio::runtime::Handle,
    data_dir: PathBuf,
) -> anyhow::Result<()> {
    let matter = MATTER.uninit().init_with(Matter::init(
        &BRIDGE_DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
        rs_matter::utils::epoch::sys_epoch,
        MATTER_PORT,
    ));

    matter.initialize_transport_buffers();

    // Persistence
    let kv_buf = KV_BUF.uninit().init_zeroed().as_mut_slice();
    let mut kv = DirKvBlobStore::new(data_dir);
    futures_lite::future::block_on(matter.load_persist(&mut kv, kv_buf))?;

    let buffers = BUFFERS.uninit().init_with(PooledBuffers::init(0));
    let subscriptions: &Subscriptions = SUBSCRIPTIONS.uninit().init_with(Subscriptions::init());

    let crypto = default_crypto(rand::thread_rng(), TEST_DEV_ATT.dac_priv_key());
    let mut rand = crypto.rand()?;

    // Build bridge handler with all devices
    let device_count = connections.len();
    let mut devices = Vec::with_capacity(device_count);
    for (i, (dk, info)) in connections.into_iter().enumerate() {
        let ep_id = 2 + i as u16;
        let device = device::Device::new(dk, rt_handle.clone());
        let bridged_info = BridgedInfo::new(Dataver::new_rand(&mut rand), &info);
        devices.push(BridgedDevice {
            ep_id,
            desc: desc::DescHandler::new(Dataver::new_rand(&mut rand)).adapt(),
            identify: StubIdentify::new(Dataver::new_rand(&mut rand)),
            bridged_info,
            on_off: onoff::OnOffHandler::new(Dataver::new_rand(&mut rand), device.clone()),
            therm: thermostat::ThermostatHandler::new(Dataver::new_rand(&mut rand), device.clone()),
            fan_ctl: fan_control::FanControlHandler::new(
                Dataver::new_rand(&mut rand),
                device.clone(),
            ),
            device,
        });
        info!("Bridged endpoint {ep_id}: {}", info.name);
    }
    let bridge = BridgeHandler { devices };
    let node = build_node(device_count);

    let events = NoEvents::new_default();

    let dm = DataModel::new(
        matter,
        &crypto,
        buffers,
        subscriptions,
        &events,
        dm_handler(rand, &bridge, node),
        SharedKvBlobStore::new(kv, kv_buf),
        SharedNetworks::new(EthNetwork::new_default()),
    );

    let responder = DefaultResponder::new(&dm);
    let mut respond = pin!(responder.run::<4, 4>());
    let mut dm_job = pin!(dm.run());

    let socket = async_io::Async::<UdpSocket>::bind(MATTER_SOCKET_BIND_ADDR)?;

    let mut mdns = pin!(run_mdns(matter));
    let mut transport = pin!(matter.run(&crypto, &socket, &socket, &socket));

    if !matter.is_commissioned() {
        matter.print_standard_qr_text(DiscoveryCapabilities::IP)?;
        matter.print_standard_qr_code(QrTextType::Unicode, DiscoveryCapabilities::IP)?;
        matter.open_basic_comm_window(MAX_COMM_WINDOW_TIMEOUT_SECS, &crypto, dm.change_notify())?;
    }

    info!("Matter stack running ({device_count} device(s))");

    let notifier = dm.change_notify();
    let mut poll = pin!(async {
        loop {
            async_io::Timer::after(Duration::from_secs(30)).await;
            for dev in &bridge.devices {
                match dev.device.get_status() {
                    Ok(_) => {
                        dev.on_off.dataver.changed();
                        dev.therm.dataver.changed();
                        dev.fan_ctl.dataver.changed();
                        notifier.notify_attr_changed(dev.ep_id, onoff::OnOffHandler::CLUSTER.id, 0);
                    }
                    Err(e) => warn!("Poll failed (ep {}): {e}", dev.ep_id),
                }
            }
            debug!("Status polled, subscriptions notified");
        }
    });

    let mut core = pin!(select4(&mut transport, &mut mdns, &mut respond, &mut dm_job).coalesce());
    let all = select(&mut core, &mut poll).coalesce();
    futures_lite::future::block_on(all)?;

    Ok(())
}
