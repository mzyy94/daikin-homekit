use clap::Parser;
use daikin_client::{Daikin, ReqwestClient};
use dsiot::protocol::property::{Binary, Metadata};
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
    println!("Checking compatibility.");
    println!("Device IP Address: {ip_addr}");

    let info = match daikin.get_info().await {
        Ok(i) => i,
        Err(error) => {
            if let Some(e) = error.downcast_ref::<reqwest::Error>() {
                println!("❌ API endpoint: Server not found. - {e}");
                return Ok(());
            }
            if let Some(e) = error.downcast_ref::<serde_json::Error>() {
                println!("❌ API endpoint: Invalid response. - {e}");
                return Ok(());
            }
            println!("❌ API endpoint: Unknown error.");
            return Ok(());
        }
    };
    println!("✅ API endpoint: available");
    println!("ℹ️  Device name: {}", info.name);
    println!("ℹ️  Device mac: {}", info.mac);
    println!("ℹ️  Device version: {}", info.version);

    let status = match daikin.get_status().await {
        Ok(s) => s,
        Err(error) => {
            if let Some(e) = error.downcast_ref::<reqwest::Error>() {
                println!("❌ Request API: Server not found. - {e}");
                return Ok(());
            }
            if let Some(e) = error.downcast_ref::<serde_json::Error>() {
                println!("❌ Request API: Invalid response. - {e}");
                return Ok(());
            }

            println!("❌ Request API: Server not found.");
            return Ok(());
        }
    };
    println!("✅ Request API: available");
    println!("✅ Status API: available");
    match &status.power.metadata {
        Metadata::Binary(Binary::Step(step)) => {
            println!(
                "ℹ️  Power Status: {:?} ({:?}) / {}",
                status.power.get_f32(),
                step.range(),
                step.step()
            );
        }
        _ => {
            println!("❌ Power Status: {:?} - invalid data", status.power);
            return Ok(());
        }
    }
    match &status.sensors.temperature.metadata {
        Metadata::Binary(Binary::Step(step)) => {
            println!(
                "ℹ️  Current temperature: {:?} ({:?}) / {}",
                status.sensors.temperature.get_f32(),
                step.range(),
                step.step()
            );
        }
        _ => {
            println!(
                "❌ Current temperature: {:?} - invalid data",
                status.sensors.temperature
            );
            return Ok(());
        }
    }
    match &status.mode.metadata {
        Metadata::Binary(Binary::Enum { max }) if max == "2F00" => {
            println!("ℹ️  Mode: {:?} [{}]", status.mode.get_enum(), max);
        }
        _ => {
            println!("❌ Mode: {:?} - invalid data", status.mode);
            return Ok(());
        }
    }
    match &status.temperature.cooling.metadata {
        Metadata::Binary(Binary::Step(step)) => {
            println!(
                "ℹ️  Target Cooling Temperature: {:?} ({:?}) / {:?}",
                status.temperature.cooling.get_f32(),
                step.range(),
                step.step()
            );
        }
        _ => {
            println!(
                "❌ Target Cooling Temperature: {:?} - invalid data",
                status.temperature.cooling
            );
            return Ok(());
        }
    }
    match &status.temperature.heating.metadata {
        Metadata::Binary(Binary::Step(step)) => {
            println!(
                "ℹ️  Target Heating Temperature: {:?} ({:?}) / {:?}",
                status.temperature.heating.get_f32(),
                step.range(),
                step.step()
            );
        }
        _ => {
            println!(
                "❌ Target Heating Temperature: {:?} - invalid data",
                status.temperature.heating
            );
            return Ok(());
        }
    }

    let mut warn = false;

    // Check cooling mode wind settings (used as reference)
    match &status.wind.cooling.speed.metadata {
        Metadata::Binary(Binary::Enum { max }) if max == "F80C" => {
            println!(
                "ℹ️  Wind Speed (Cooling): {:?} [{}]",
                status.wind.cooling.speed.get_enum(),
                max
            );
        }
        _ => {
            println!(
                "⚠️  Wind Speed (Cooling): {:?} - invalid data",
                status.wind.cooling.speed
            );
            warn = true;
        }
    }
    match &status.wind.cooling.vertical_direction.metadata {
        Metadata::Binary(Binary::Enum { max }) if max == "3F808100" => {
            println!(
                "ℹ️  Vertical Wind Direction (Cooling): {:?} [{}]",
                status.wind.cooling.vertical_direction.get_enum(),
                max
            );
        }
        _ => {
            println!(
                "⚠️  Vertical Wind Direction (Cooling): {:?} - invalid data",
                status.wind.cooling.vertical_direction
            );
            warn = true;
        }
    }
    match &status.wind.cooling.horizontal_direction.metadata {
        Metadata::Binary(Binary::Enum { max }) if max == "FD8101" => {
            println!(
                "ℹ️  Horizontal Wind Direction (Cooling): {:?} [{}]",
                status.wind.cooling.horizontal_direction.get_enum(),
                max
            );
        }
        _ => {
            println!(
                "⚠️  Horizontal Wind Direction (Cooling): {:?} - invalid data",
                status.wind.cooling.horizontal_direction
            );
            warn = true;
        }
    }

    if warn {
        println!("🙆 Your device is mostly supported except optional wind control.");
    } else {
        println!("🎉 Your device is perfectly compatible.");
    }

    Ok(())
}
