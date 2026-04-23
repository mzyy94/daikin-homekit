#[macro_use]
extern crate log;

mod bridge;
mod bridged_info;
mod device;
mod fan_control;
mod identify;
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
use rs_matter::dm::clusters::desc::{self, ClusterHandler as _};
use rs_matter::dm::clusters::dev_att::DeviceAttestation;
use rs_matter::dm::clusters::net_comm::SharedNetworks;
use rs_matter::dm::devices::test::{TEST_DEV_ATT, TEST_DEV_COMM};
use rs_matter::dm::events::NoEvents;
use rs_matter::dm::networks::eth::EthNetwork;
use rs_matter::dm::networks::unix::UnixNetifs;
use rs_matter::dm::subscriptions::Subscriptions;
use rs_matter::dm::{
    Async, DataModel, Dataver, EmptyHandler, EpClMatcher, IMBuffer, Node, endpoints,
};
use rs_matter::error::Error;
use rs_matter::pairing::{DiscoveryCapabilities, qr::QrTextType};
use rs_matter::persist::{DirKvBlobStore, SharedKvBlobStore};
use rs_matter::respond::DefaultResponder;
use rs_matter::sc::pase::MAX_COMM_WINDOW_TIMEOUT_SECS;
use rs_matter::transport::MATTER_SOCKET_BIND_ADDR;
use rs_matter::utils::init::InitMaybeUninit;
use rs_matter::utils::select::Coalesce;
use rs_matter::utils::storage::pooled::PooledBuffers;
use rs_matter::{MATTER_PORT, Matter};

use bridge::{BridgeHandler, BridgedMatcher};

static MATTER: StaticCell<Matter> = StaticCell::new();
static BUFFERS: StaticCell<PooledBuffers<10, IMBuffer>> = StaticCell::new();
static SUBSCRIPTIONS: StaticCell<Subscriptions> = StaticCell::new();
static KV_BUF: StaticCell<[u8; 4096]> = StaticCell::new();

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

    let kv_buf = KV_BUF.uninit().init_zeroed().as_mut_slice();
    let mut kv = DirKvBlobStore::new(data_dir);
    futures_lite::future::block_on(matter.load_persist(&mut kv, kv_buf))?;

    let buffers = BUFFERS.uninit().init_with(PooledBuffers::init(0));
    let subscriptions: &Subscriptions = SUBSCRIPTIONS.uninit().init_with(Subscriptions::init());

    let crypto = default_crypto(rand::thread_rng(), TEST_DEV_ATT.dac_priv_key());
    let mut rand = crypto.rand()?;

    let device_count = connections.len();
    let mut devices = Vec::with_capacity(device_count);
    for (i, (dk, info)) in connections.into_iter().enumerate() {
        let ep_id = 2 + i as u16;
        let device = device::Device::new(dk, rt_handle.clone());
        let bridged_info = bridged_info::BridgedInfo::new(Dataver::new_rand(&mut rand), &info);
        devices.push(bridge::BridgedDevice::new(
            ep_id,
            &mut rand,
            bridged_info,
            device,
        ));
        info!("Bridged endpoint {ep_id}: {}", info.name);
    }
    let bridge_handler = BridgeHandler { devices };
    let node = bridge::build_node(device_count);

    let events = NoEvents::new_default();

    let dm = DataModel::new(
        matter,
        &crypto,
        buffers,
        subscriptions,
        &events,
        dm_handler(rand, &bridge_handler, node),
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
            for dev in &bridge_handler.devices {
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
