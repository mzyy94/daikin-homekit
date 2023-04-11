mod daikin;
#[macro_use]
mod response;
mod info;
#[macro_use]
mod request;
mod property;
mod status;

use crate::daikin::Daikin;
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

    let mut status = daikin.get_status().await?;
    println!("{:#?}", status);

    status.set_power(false);
    status.set_mode(status::Mode::Cooling);
    status.set_target_cooling_temperature(28.0);
    status.set_target_heating_temperature(24.0);
    status.set_target_automatic_temperature(-1.0);
    daikin.update(status).await?;

    Ok(())
}
