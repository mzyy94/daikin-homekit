use clap::Parser;
use daikin_homekit::daikin::Daikin;
use daikin_homekit::error::Error;
use daikin_homekit::status;
use std::net::Ipv4Addr;

#[derive(Parser)]
#[clap(
    author = "mzyy94",
    version = "v0.0.1",
    about = "Get current status from Daikin AC"
)]
struct Cli {
    /// IPv4 address of Daikin AC
    #[arg(value_name = "ip_address")]
    ip_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let addr = cli.ip_addr.parse::<Ipv4Addr>()?;
    get_status(addr).await
}

async fn get_status(ip_addr: Ipv4Addr) -> Result<(), Error> {
    let daikin = Daikin::new(ip_addr);

    let info = daikin.get_info().await?;
    println!("{:#?}", info);

    let mut status = daikin.get_status().await?;
    println!("{:#?}", status);

    status.set_power(false).ok();
    status.set_mode(status::Mode::Cooling).ok();
    status.set_target_cooling_temperature(28.0).ok();
    status.set_target_heating_temperature(24.0).ok();
    status.set_target_automatic_temperature(-1.0).ok();
    daikin.update(status).await?;

    Ok(())
}
