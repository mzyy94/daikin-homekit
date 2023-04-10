mod daikin;
mod response;
mod status;

use crate::daikin::Daikin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1) {
        Some(ip_addr) => get_status(ip_addr).await,
        None => Ok(()),
    }
}

async fn get_status(ip_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    let daikin = Daikin::new(ip_addr);
    let status = daikin.get_status().await?;
    println!("{:#?}", status);

    Ok(())
}
