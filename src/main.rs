mod daikin;
#[macro_use]
mod response;
mod info;
#[macro_use]
mod request;
mod property;
mod status;

use crate::{daikin::Daikin, status::DaikinStatus};
use std::net::Ipv4Addr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1) {
        Some(ip_addr) => {
            let addr = ip_addr.parse::<Ipv4Addr>()?;
            get_status(addr).await
        }
        None => Ok(()),
    }
}

async fn get_status(ip_addr: Ipv4Addr) -> Result<(), Box<dyn std::error::Error>> {
    let daikin = Daikin::new(ip_addr);

    let info = daikin.get_info().await?;
    println!("{:#?}", info);

    let status = daikin.get_status().await?;
    println!("{:#?}", status);

    let mut status = DaikinStatus::new();
    status.set_power(false);
    let j = serde_json::to_string_pretty(&status)?;
    println!("{}", j);

    Ok(())
}
