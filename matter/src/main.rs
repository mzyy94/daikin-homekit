#[macro_use]
extern crate log;

mod device;
mod onoff;

use core::pin::pin;
use std::net::{Ipv4Addr, UdpSocket};

use clap::Parser;
use daikin_client::{Daikin, ReqwestClient};
use embassy_futures::select::select4;
use static_cell::StaticCell;

use rs_matter::crypto::{Crypto, default_crypto};
use rs_matter::dm::clusters::basic_info::BasicInfoConfig;
use rs_matter::dm::clusters::decl::{fan_control, identify, on_off, thermostat};
use rs_matter::dm::clusters::desc::{self, ClusterHandler as _};
use rs_matter::dm::clusters::dev_att::DeviceAttestation;
use rs_matter::dm::clusters::net_comm::SharedNetworks;
use rs_matter::dm::devices::test::{TEST_DEV_ATT, TEST_DEV_COMM};
use rs_matter::dm::events::NoEvents;
use rs_matter::dm::networks::eth::EthNetwork;
use rs_matter::dm::networks::unix::UnixNetifs;
use rs_matter::dm::subscriptions::Subscriptions;
use rs_matter::dm::{
    Async, Cluster, DataModel, Dataver, DeviceType, EmptyHandler, Endpoint, EpClMatcher, IMBuffer,
    InvokeContext, Node, ReadContext, WriteContext, endpoints,
};
use rs_matter::error::{Error, ErrorCode};
use rs_matter::im::Percent;
use rs_matter::pairing::{DiscoveryCapabilities, qr::QrTextType};
use rs_matter::persist::{DirKvBlobStore, SharedKvBlobStore};
use rs_matter::respond::DefaultResponder;
use rs_matter::sc::pase::MAX_COMM_WINDOW_TIMEOUT_SECS;
use rs_matter::tlv::{Nullable, TLVBuilderParent};
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

const DEV_DET: BasicInfoConfig<'static> = BasicInfoConfig {
    vid: 0xfff1,
    pid: 0x8001,
    product_name: "Daikin Air Conditioner",
    vendor_name: "Daikin",
    device_name: "DaikinAC",
    hw_ver: 1,
    hw_ver_str: "1",
    sw_ver: 1,
    sw_ver_str: "0.1.0",
    serial_no: "000000000000",
    ..BasicInfoConfig::new()
};

const NODE: Node<'static> = Node {
    endpoints: &[
        root_endpoint!(geth),
        Endpoint {
            id: 1,
            device_types: devices!(DEV_TYPE_ROOM_AC),
            clusters: clusters!(
                desc::DescHandler::CLUSTER,
                StubIdentify::CLUSTER,
                onoff::OnOffHandler::CLUSTER,
                StubThermostat::CLUSTER,
                StubFanControl::CLUSTER
            ),
        },
    ],
};

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

struct StubThermostat {
    dataver: Dataver,
}

impl StubThermostat {
    const CLUSTER: Cluster<'static> = thermostat::FULL_CLUSTER
        .with_revision(7)
        .with_features(
            thermostat::Feature::HEATING.bits()
                | thermostat::Feature::COOLING.bits()
                | thermostat::Feature::AUTO_MODE.bits(),
        )
        .with_attrs(with!(
            required;
            thermostat::AttributeId::LocalTemperature
            | thermostat::AttributeId::SystemMode
            | thermostat::AttributeId::OccupiedCoolingSetpoint
            | thermostat::AttributeId::OccupiedHeatingSetpoint
            | thermostat::AttributeId::ControlSequenceOfOperation
        ));

    fn new(dataver: Dataver) -> Self {
        Self { dataver }
    }
}

impl thermostat::ClusterHandler for StubThermostat {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn local_temperature(&self, _ctx: impl ReadContext) -> Result<Nullable<i16>, Error> {
        Ok(Nullable::some(2500)) // 25.0°C
    }

    fn system_mode(&self, _ctx: impl ReadContext) -> Result<thermostat::SystemModeEnum, Error> {
        Ok(thermostat::SystemModeEnum::Off)
    }

    fn set_system_mode(
        &self,
        _ctx: impl WriteContext,
        _value: thermostat::SystemModeEnum,
    ) -> Result<(), Error> {
        debug!("Thermostat: set system_mode (stub)");
        Ok(())
    }

    fn occupied_cooling_setpoint(&self, _ctx: impl ReadContext) -> Result<i16, Error> {
        Ok(2600) // 26.0°C
    }

    fn set_occupied_cooling_setpoint(
        &self,
        _ctx: impl WriteContext,
        _value: i16,
    ) -> Result<(), Error> {
        debug!("Thermostat: set cooling setpoint (stub)");
        Ok(())
    }

    fn occupied_heating_setpoint(&self, _ctx: impl ReadContext) -> Result<i16, Error> {
        Ok(2000) // 20.0°C
    }

    fn set_occupied_heating_setpoint(
        &self,
        _ctx: impl WriteContext,
        _value: i16,
    ) -> Result<(), Error> {
        debug!("Thermostat: set heating setpoint (stub)");
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

struct StubFanControl {
    dataver: Dataver,
}

impl StubFanControl {
    const CLUSTER: Cluster<'static> = fan_control::FULL_CLUSTER
        .with_revision(4)
        .with_features(
            fan_control::Feature::MULTI_SPEED.bits()
                | fan_control::Feature::AUTO.bits()
                | fan_control::Feature::ROCKING.bits()
                | fan_control::Feature::WIND.bits()
                | fan_control::Feature::STEP.bits(),
        )
        .with_attrs(with!(
            required;
            fan_control::AttributeId::FanMode
            | fan_control::AttributeId::FanModeSequence
            | fan_control::AttributeId::SpeedSetting
            | fan_control::AttributeId::SpeedMax
            | fan_control::AttributeId::SpeedCurrent
            | fan_control::AttributeId::RockSupport
            | fan_control::AttributeId::RockSetting
            | fan_control::AttributeId::WindSupport
            | fan_control::AttributeId::WindSetting
        ));

    fn new(dataver: Dataver) -> Self {
        Self { dataver }
    }
}

impl fan_control::ClusterHandler for StubFanControl {
    const CLUSTER: Cluster<'static> = Self::CLUSTER;

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }
    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn fan_mode(&self, _ctx: impl ReadContext) -> Result<fan_control::FanModeEnum, Error> {
        Ok(fan_control::FanModeEnum::Auto)
    }

    fn fan_mode_sequence(
        &self,
        _ctx: impl ReadContext,
    ) -> Result<fan_control::FanModeSequenceEnum, Error> {
        Ok(fan_control::FanModeSequenceEnum::OffLowMedHighAuto)
    }

    fn percent_setting(&self, _ctx: impl ReadContext) -> Result<Nullable<Percent>, Error> {
        Ok(Nullable::some(0))
    }

    fn percent_current(&self, _ctx: impl ReadContext) -> Result<Percent, Error> {
        Ok(0)
    }

    fn speed_max(&self, _ctx: impl ReadContext) -> Result<u8, Error> {
        Ok(5)
    }

    fn speed_setting(&self, _ctx: impl ReadContext) -> Result<Nullable<u8>, Error> {
        Ok(Nullable::some(0))
    }

    fn speed_current(&self, _ctx: impl ReadContext) -> Result<u8, Error> {
        Ok(0)
    }

    fn rock_support(&self, _ctx: impl ReadContext) -> Result<fan_control::RockBitmap, Error> {
        Ok(fan_control::RockBitmap::ROCK_UP_DOWN | fan_control::RockBitmap::ROCK_LEFT_RIGHT)
    }

    fn rock_setting(&self, _ctx: impl ReadContext) -> Result<fan_control::RockBitmap, Error> {
        Ok(fan_control::RockBitmap::empty())
    }

    fn wind_support(&self, _ctx: impl ReadContext) -> Result<fan_control::WindBitmap, Error> {
        Ok(fan_control::WindBitmap::SLEEP_WIND | fan_control::WindBitmap::NATURAL_WIND)
    }

    fn wind_setting(&self, _ctx: impl ReadContext) -> Result<fan_control::WindBitmap, Error> {
        Ok(fan_control::WindBitmap::empty())
    }

    fn set_fan_mode(
        &self,
        _ctx: impl WriteContext,
        _value: fan_control::FanModeEnum,
    ) -> Result<(), Error> {
        debug!("FanControl: set fan_mode (stub)");
        Ok(())
    }

    fn set_percent_setting(
        &self,
        _ctx: impl WriteContext,
        _value: Nullable<Percent>,
    ) -> Result<(), Error> {
        debug!("FanControl: set percent_setting (stub)");
        Ok(())
    }

    fn handle_step(
        &self,
        _ctx: impl InvokeContext,
        _req: fan_control::StepRequest<'_>,
    ) -> Result<(), Error> {
        Err(ErrorCode::InvalidCommand.into())
    }
}

fn dm_handler<'a>(
    mut rand: impl rand::RngCore,
    identify: &'a StubIdentify,
    on_off: &'a onoff::OnOffHandler,
    thermostat: &'a StubThermostat,
    fan_control: &'a StubFanControl,
) -> impl rs_matter::dm::AsyncMetadata + rs_matter::dm::AsyncHandler + 'a {
    let desc_dataver = Dataver::new_rand(&mut rand);

    (
        NODE,
        endpoints::with_eth_sys(
            &false,
            &(),
            &UnixNetifs,
            rand,
            EmptyHandler
                .chain(
                    EpClMatcher::new(Some(1), Some(desc::DescHandler::CLUSTER.id)),
                    Async(desc::DescHandler::new(desc_dataver).adapt()),
                )
                .chain(
                    EpClMatcher::new(Some(1), Some(StubIdentify::CLUSTER.id)),
                    Async(identify::HandlerAdaptor(identify)),
                )
                .chain(
                    EpClMatcher::new(Some(1), Some(onoff::OnOffHandler::CLUSTER.id)),
                    Async(on_off::HandlerAdaptor(on_off)),
                )
                .chain(
                    EpClMatcher::new(Some(1), Some(StubThermostat::CLUSTER.id)),
                    Async(thermostat::HandlerAdaptor(thermostat)),
                )
                .chain(
                    EpClMatcher::new(Some(1), Some(StubFanControl::CLUSTER.id)),
                    Async(fan_control::HandlerAdaptor(fan_control)),
                ),
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
    /// IP address of the Daikin device
    ip_addr: Ipv4Addr,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("daikin_matter=debug,rs_matter=info"),
    )
    .init();

    let cli = Cli::parse();

    // Initialize daikin-client on tokio
    let rt = tokio::runtime::Runtime::new()?;
    let dk = rt.block_on(async {
        let dk = Daikin::new(cli.ip_addr, ReqwestClient::try_new()?);
        let info = dk.get_info().await?;
        info!(
            "Device: {} (MAC: {}, EDID: {})",
            info.name, info.mac, info.edid
        );
        let status = dk.get_status().await?;
        debug!("Status: {:?}", status);
        anyhow::Ok(dk)
    })?;

    let rt_handle = rt.handle().clone();

    // Run Matter stack on a separate thread with increased stack
    let thread = std::thread::Builder::new()
        .stack_size(550 * 1024)
        .spawn(move || run_matter(dk, rt_handle))
        .unwrap();

    thread.join().unwrap()
}

fn run_matter(dk: Daikin<ReqwestClient>, rt_handle: tokio::runtime::Handle) -> anyhow::Result<()> {
    let matter = MATTER.uninit().init_with(Matter::init(
        &DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
        rs_matter::utils::epoch::sys_epoch,
        MATTER_PORT,
    ));

    matter.initialize_transport_buffers();

    // Persistence
    let kv_buf = KV_BUF.uninit().init_zeroed().as_mut_slice();
    let mut kv = DirKvBlobStore::new_default();
    futures_lite::future::block_on(matter.load_persist(&mut kv, kv_buf))?;

    let buffers = BUFFERS.uninit().init_with(PooledBuffers::init(0));
    let subscriptions = SUBSCRIPTIONS.uninit().init_with(Subscriptions::init());

    let crypto = default_crypto(rand::thread_rng(), TEST_DEV_ATT.dac_priv_key());
    let mut rand = crypto.rand()?;

    let device = device::Device::new(dk, rt_handle);

    // Create handlers
    let identify = StubIdentify::new(Dataver::new_rand(&mut rand));
    let on_off = onoff::OnOffHandler::new(Dataver::new_rand(&mut rand), device.clone());
    let thermostat = StubThermostat::new(Dataver::new_rand(&mut rand));
    let fan_control = StubFanControl::new(Dataver::new_rand(&mut rand));

    let events = NoEvents::new_default();

    let dm = DataModel::new(
        matter,
        &crypto,
        buffers,
        subscriptions,
        &events,
        dm_handler(rand, &identify, &on_off, &thermostat, &fan_control),
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

    info!("Matter stack running");

    let all = select4(&mut transport, &mut mdns, &mut respond, &mut dm_job).coalesce();
    futures_lite::future::block_on(all)?;

    Ok(())
}
