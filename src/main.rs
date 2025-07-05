use clap::{crate_authors, crate_description, crate_name, crate_version, Parser};
use daikin_homekit::{characteristic::setup_characteristic, daikin::Daikin};
use futures::{pin_mut, prelude::*};
use log::{info, warn};
use std::{net::Ipv4Addr, str::FromStr};

use hap::{
    accessory::{
        bridge::BridgeAccessory, heater_cooler::HeaterCoolerAccessory, AccessoryCategory,
        AccessoryInformation,
    },
    server::{IpServer, Server},
    storage::{FileStorage, Storage},
    Config, MacAddress, Pin,
};

#[derive(Parser)]
#[clap(
    name = crate_name!(),
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
)]
struct Cli {
    /// IPv4 address of Daikin AC
    #[arg(value_name = "ip_address")]
    ip_addr: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "hap=info,daikin_homekit=debug");
    }
    env_logger::init();

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
            let stream = Daikin::discovery(timeout).await?;
            pin_mut!(stream);
            stream
                .next()
                .await
                .ok_or(daikin_homekit::error::Error::Unknown)??
        }
    };

    let bridge = BridgeAccessory::new(
        1,
        AccessoryInformation {
            name: "Daikin Bridge".into(),
            manufacturer: crate_authors!().into(),
            model: crate_name!().into(),
            serial_number: "000000000000".into(),
            ..Default::default()
        },
    )?;

    let mut storage = {
        if cfg!(debug_assertions) {
            FileStorage::current_dir().await?
        } else if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("daikin-homekit");
            FileStorage::new(&config_dir).await?
        } else {
            warn!("could not detect config directory. falling back to current dir");
            FileStorage::current_dir().await?
        }
    };

    info!("config file location: {:?}", storage);

    let config = match storage.load_config().await {
        Ok(mut config) => {
            config.redetermine_local_ip();
            storage.save_config(&config).await?;
            config
        }
        Err(_) => {
            let config = Config {
                pin: Pin::new([2, 0, 2, 3, 0, 4, 2, 0])?,
                name: "Daikin Bridge".into(),
                device_id: MacAddress::from_str(&info.mac().unwrap_or("000000000000".into()))
                    .unwrap(),
                category: AccessoryCategory::AirConditioner,
                ..Default::default()
            };
            storage.save_config(&config).await?;
            config
        }
    };

    let server = IpServer::new(config, storage).await?;
    server.add_accessory(bridge).await?;

    let mut ac = HeaterCoolerAccessory::new(
        info.edid().unwrap_or(2),
        AccessoryInformation {
            name: info.name().unwrap_or("Unknown name".into()),
            manufacturer: "Daikin Industries, Ltd.".into(),
            serial_number: info.mac().unwrap_or("000000000000".into()),
            // WARNING: DO NOT COMMENT OUT BELOW
            // firmware_revision: info.version(),
            ..Default::default()
        },
    )?;
    setup_characteristic(daikin, &mut ac.heater_cooler).await?;
    server.add_accessory(ac).await?;

    let handle = server.run_handle();

    handle.await?;
    Ok(())
}
