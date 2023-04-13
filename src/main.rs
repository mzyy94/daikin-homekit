use clap::Parser;
use daikin_homekit::error::Error;
use daikin_homekit::{
    characteristic::{set_initial_value, setup_characteristic_callback},
    daikin::Daikin,
};
use std::{net::Ipv4Addr, str::FromStr};

use hap::{
    accessory::{heater_cooler::HeaterCoolerAccessory, AccessoryCategory, AccessoryInformation},
    server::{IpServer, Server},
    storage::{FileStorage, Storage},
    Config, MacAddress, Pin,
};

#[derive(Parser)]
#[clap(
    author = "mzyy94",
    version = "v0.0.1",
    about = "Control Daikin AC via HomeKit"
)]
struct Cli {
    /// IPv4 address of Daikin AC
    #[arg(value_name = "ip_address")]
    ip_addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let (daikin, info) = match cli.ip_addr {
        Some(ip_addr) => {
            let ip_addr = ip_addr.parse::<Ipv4Addr>()?;
            let daikin = Daikin::new(ip_addr);

            let info = daikin.get_info().await?;
            (daikin, info)
        }
        None => {
            let timeout = std::time::Duration::new(3, 0);
            Daikin::discovery(timeout).await.unwrap()
        }
    };

    let mut ac = HeaterCoolerAccessory::new(
        1,
        AccessoryInformation {
            name: info.name().unwrap_or("Unknown name".into()),
            manufacturer: "Daikin Industries, Ltd.".into(),
            serial_number: info.mac().unwrap_or("000000000000".into()),
            // WARNING: DO NOT COMMENT OUT BELOW
            // firmware_revision: info.version(),
            ..Default::default()
        },
    )?;

    let mut storage = FileStorage::current_dir().await?;

    let config = match storage.load_config().await {
        Ok(mut config) => {
            config.redetermine_local_ip();
            storage.save_config(&config).await?;
            config
        }
        Err(_) => {
            let config = Config {
                pin: Pin::new([1, 1, 1, 2, 2, 3, 3, 3])?,
                name: info.name().unwrap_or("Daikin AC".into()),
                device_id: MacAddress::from_str(&info.mac().unwrap_or("000000000000".into()))
                    .unwrap(),
                category: AccessoryCategory::AirConditioner,
                ..Default::default()
            };
            storage.save_config(&config).await?;
            config
        }
    };

    ac.heater_cooler.lock_physical_controls = None;
    ac.heater_cooler.name = None;
    ac.heater_cooler.temperature_display_units = None;

    let status = daikin.get_status().await?;
    set_initial_value(status, &mut ac.heater_cooler).await?;
    setup_characteristic_callback(daikin, &mut ac.heater_cooler);

    let server = IpServer::new(config, storage).await?;
    server.add_accessory(ac).await?;

    let handle = server.run_handle();

    std::env::set_var("RUST_LOG", "hap=debug");
    env_logger::init();

    handle.await?;
    Ok(())
}
