use clap::Parser;
use daikin_homekit::client::ReqwestClient;
use daikin_homekit::daikin::Daikin;
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
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let addr = cli.ip_addr.parse::<Ipv4Addr>()?;
    get_status(addr).await
}

async fn get_status(ip_addr: Ipv4Addr) -> anyhow::Result<()> {
    let client = ReqwestClient::try_new()?;
    let daikin = Daikin::new(ip_addr, client);

    let info = daikin.get_info().await?;
    println!("{info:#?}");

    let status = daikin.get_status().await?;
    println!("{status:#?}");

    Ok(())
}
