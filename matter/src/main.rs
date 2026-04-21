#[macro_use]
extern crate log;

use clap::Parser;
use daikin_client::{Daikin, ReqwestClient};
use std::net::Ipv4Addr;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// IP address of the Daikin device
    ip_addr: Ipv4Addr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("daikin_matter=debug,rs_matter=info"),
    )
    .init();

    let cli = Cli::parse();

    info!("Connecting to Daikin device at {}", cli.ip_addr);
    let daikin = Daikin::new(cli.ip_addr, ReqwestClient::try_new()?);

    let info = daikin.get_info().await?;
    info!(
        "Device: {} (MAC: {}, EDID: {})",
        info.name, info.mac, info.edid
    );

    let status = daikin.get_status().await?;
    debug!("Status: {:?}", status);

    Ok(())
}
