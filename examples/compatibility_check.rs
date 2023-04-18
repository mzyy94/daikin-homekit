use clap::Parser;
use daikin_homekit::daikin::Daikin;
use daikin_homekit::error::Error;
use daikin_homekit::status::Meta;
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
    let daikin = Daikin::new(ip_addr);
    println!("Checking compatibility.");
    println!("Device IP Address: {}", ip_addr);

    let info = match daikin.get_info().await {
        Ok(i) => i,
        Err(error) => {
            if let Some(e) = error.downcast_ref::<reqwest::Error>() {
                println!("❌ API endpoint: Server not found. - {}", e);
                return Ok(());
            }
            if let Some(e) = error.downcast_ref::<Error>() {
                println!("❌ API endpoint: HTTP Request failed. - {}", e);
                return Ok(());
            }
            if let Some(e) = error.downcast_ref::<serde_json::Error>() {
                println!("❌ API endpoint: Invalid response. - {}", e);
                return Ok(());
            }
            println!("❌ API endpoint: Unknown error.");
            return Ok(());
        }
    };
    println!("✅ API endpoint: available");
    println!("ℹ️  Device name: {}", info.name().unwrap_or_default());
    println!("ℹ️  Device mac: {}", info.mac().unwrap_or_default());
    println!("ℹ️  Device version: {}", info.version().unwrap_or_default());

    let status = match daikin.get_status().await {
        Ok(s) => s,
        Err(error) => {
            if let Some(e) = error.downcast_ref::<reqwest::Error>() {
                println!("❌ Request API: Server not found. - {}", e);
                return Ok(());
            }
            if let Some(e) = error.downcast_ref::<Error>() {
                match e {
                    Error::RSCError(e) => {
                        println!("✅ Request API: available");
                        println!("❌ Status API: unavailable - {:?}", e);
                    }
                    _ => println!("❌ Request API: HTTP Request failed. - {}", e),
                };
                return Ok(());
            }
            if let Some(e) = error.downcast_ref::<serde_json::Error>() {
                println!("❌ Request API: Invalid response. - {}", e);
                return Ok(());
            }

            println!("❌ Request API: Server not found.");
            return Ok(());
        }
    };
    println!("✅ Request API: available");
    println!("✅ Status API: available");
    match status.power {
        Some(v) => {
            if let Meta {
                step,
                min: Some(min),
                max: Some(max),
                digits: _,
            } = status.meta.power
            {
                println!("ℹ️  Power Status: {v:?} ({min:?} .. {max:?}) / {step:?}");
            } else {
                println!("❌  Power Status: {v:?} - invalid metadata");
                return Ok(());
            }
        }
        None => {
            println!("❌ Power Status: unavailable.");
            return Ok(());
        }
    }
    match status.current_temperature {
        Some(v) => {
            if let Meta {
                step,
                min: Some(min),
                max: Some(max),
                digits: _,
            } = status.meta.current_temperature
            {
                println!("ℹ️  Current temperature: {v:?} ({min:?} .. {max:?}) / {step:?}");
            } else {
                println!("❌  Current temperature: {v:?} - invalid metadata");
                return Ok(());
            }
        }
        None => {
            println!("❌ Current temperature: unavailable.");
            return Ok(());
        }
    }
    match status.mode {
        Some(v) => {
            if let Meta {
                step: _,
                min: None,
                max: Some(max),
                digits: _,
            } = status.meta.mode
            {
                if max as u32 == 0x002f {
                    println!("ℹ️  Mode: {:?} [0x{:04x}]", v, max as u32);
                } else {
                    println!(
                        "❌  Mode: {:?} [0x{:04x}] - invalid metadata",
                        v, max as u32
                    );
                    return Ok(());
                }
            } else {
                println!("❌  Mode: {v:?} - invalid metadata");
                return Ok(());
            }
        }
        None => {
            println!("❌ Mode: unavailable.");
            return Ok(());
        }
    }
    match status.target_cooling_temperature {
        Some(v) => {
            if let Meta {
                step,
                min: Some(min),
                max: Some(max),
                digits: _,
            } = status.meta.target_cooling_temperature
            {
                println!("ℹ️  Target Cooling Temperature: {v:?} ({min:?} .. {max:?}) / {step:?}");
            } else {
                println!("❌  Target Cooling Temperature: {v:?} - invalid metadata");
                return Ok(());
            }
        }
        None => {
            println!("❌ Target Cooling Temperature: unavailable.");
            return Ok(());
        }
    }
    match status.target_heating_temperature {
        Some(v) => {
            if let Meta {
                step,
                min: Some(min),
                max: Some(max),
                digits: _,
            } = status.meta.target_heating_temperature
            {
                println!("ℹ️  Target Heating Temperature: {v:?} ({min:?} .. {max:?}) / {step:?}");
            } else {
                println!("❌  Target Heating Temperature: {v:?} - invalid metadata");
                return Ok(());
            }
        }
        None => {
            println!("❌ Target Heating Temperature: unavailable.");
            return Ok(());
        }
    }

    let mut warn = false;

    match status.wind_speed {
        Some(v) => {
            if let Meta {
                step: _,
                min: None,
                max: Some(max),
                digits: _,
            } = status.meta.wind_speed
            {
                if max as u32 == 0x0cf8 {
                    println!("ℹ️  Wind Speed: {:?} [0x{:04x}]", v, max as u32);
                } else {
                    println!(
                        "⚠️  Wind Speed: {:?} [0x{:04x}] - invalid metadata",
                        v, max as u32
                    );
                    warn = true;
                }
            } else {
                println!("⚠️  Wind Speed: {v:?} - invalid metadata");
                warn = true;
            }
        }
        None => {
            println!("⚠️  Wind Speed: unavailable.");
            warn = true;
        }
    }
    match status.vertical_wind_direction {
        Some(v) => {
            if let Meta {
                step: _,
                min: None,
                max: Some(max),
                digits: _,
            } = status.meta.vertical_wind_direction
            {
                if max as u32 == 0x0081803f {
                    println!("ℹ️  Vertical Wind Direction: {:?} [0x{:08x}]", v, max as u32);
                } else {
                    println!(
                        "⚠️  Vertical Wind Direction: {:?} [0x{:08x}] - invalid metadata",
                        v, max as u32
                    );
                    warn = true;
                }
            } else {
                println!("⚠️  Vertical Wind Direction: {v:?} - invalid metadata");
                warn = true;
            }
        }
        None => {
            println!("⚠️  Vertical Wind Direction: unavailable.");
            warn = true;
        }
    }
    match status.horizontal_wind_direction {
        Some(v) => {
            if let Meta {
                step: _,
                min: None,
                max: Some(max),
                digits: _,
            } = status.meta.horizontal_wind_direction
            {
                if max as u32 == 0x0181fd {
                    println!(
                        "ℹ️  Horizontal Wind Direction: {:?} [0x{:06x}]",
                        v, max as u32
                    );
                } else {
                    println!(
                        "⚠️  Horizontal Wind Direction: {:?} [0x{:06x}] - invalid metadata",
                        v, max as u32
                    );
                    warn = true;
                }
            } else {
                println!("⚠️  Horizontal Wind Direction: {v:?} - invalid metadata");
                warn = true;
            }
        }
        None => {
            println!("⚠️  Horizontal Wind Direction: unavailable.");
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
