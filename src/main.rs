use clap::{crate_authors, crate_name, Parser};
use daikin_homekit::{
    characteristic::setup_characteristic, daikin::Daikin, discovery::discovery, info::DaikinInfo,
};
use futures::prelude::*;
use log::{error, info, warn};
use std::net::Ipv4Addr;

use hap::{
    accessory::{
        bridge::BridgeAccessory, heater_cooler::HeaterCoolerAccessory, AccessoryCategory,
        AccessoryInformation,
    },
    server::{IpServer, Server},
    storage::{FileStorage, Storage},
    Config, Pin,
};

#[derive(Parser)]
#[command(name = crate_name!(), author, version, about)]
struct Cli {
    /// IPv4 address of Daikin AC
    #[arg(value_name = "ip_address", exclusive = true)]
    ip_addrs: Vec<Ipv4Addr>,
    /// Discovery timeout in milliseconds
    #[arg(long, default_value = "3000")]
    timeout: u64,
    /// Expected number of devices to discover
    #[arg(long, default_value = "128", hide_default_value = true)]
    count: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "hap=info,daikin_homekit=debug");
    }
    env_logger::init();

    let cli = Cli::parse();

    let devices: Vec<(Daikin, DaikinInfo)> = if !cli.ip_addrs.is_empty() {
        info!("Using provided IP address(es): {:?}", cli.ip_addrs);

        stream::iter(cli.ip_addrs)
            .then(|ip| async move {
                let daikin = Daikin::new(ip);
                let info = daikin.get_info().await?;
                anyhow::Ok((daikin, info))
            })
            .try_collect::<Vec<_>>()
            .await?
    } else {
        warn!("No IP address provided. Discovering Daikin devices on the local network...");
        let timeout = std::time::Duration::from_millis(cli.timeout);
        let stream = discovery(timeout).await;
        let devices = stream.take(cli.count).try_collect::<Vec<_>>().await?;
        if devices.is_empty() {
            error!("No Daikin devices found.");
            return Ok(());
        }
        if cli.count != 128 && devices.len() < cli.count {
            error!(
                "Found only {} devices, but requested {}.",
                devices.len(),
                cli.count
            );
            return Ok(());
        }
        devices
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
                category: AccessoryCategory::AirConditioner,
                ..Default::default()
            };
            storage.save_config(&config).await?;
            config
        }
    };

    let server = IpServer::new(config, storage).await?;
    server.add_accessory(bridge).await?;

    for (daikin, info) in &devices {
        let mut ac = HeaterCoolerAccessory::new(
            info.edid,
            AccessoryInformation {
                name: info.name.clone(),
                manufacturer: "Daikin Industries, Ltd.".into(),
                serial_number: info.mac.clone(),
                // WARNING: DO NOT COMMENT OUT BELOW
                // firmware_revision: info.version(),
                ..Default::default()
            },
        )?;
        setup_characteristic(daikin.clone(), &mut ac.heater_cooler).await?;
        server.add_accessory(ac).await?;
    }

    let handle = server.run_handle();

    handle.await?;
    Ok(())
}
