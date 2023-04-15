use crate::daikin::Daikin;
use crate::info::DaikinInfo;
use futures::prelude::*;
use genawaiter::sync::gen;
use genawaiter::yield_;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;
use std::time::Duration;
use tokio::net::UdpSocket;

fn get_ipaddr() -> (IpAddr, IpAddr) {
    let network_interfaces = match NetworkInterface::show() {
        Ok(i) => i,
        Err(_) => {
            return (
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
            )
        }
    };

    let nis: Vec<&NetworkInterface> = network_interfaces
        .iter()
        .filter(|ni| {
            ni.mac_addr != Some("00:00:00:00:00:00".into())
                && ni.mac_addr != None
                && ni.addr.iter().filter(|a| a.ip().is_ipv4()).count() > 0
        })
        .collect();

    if nis.len() != 1 {
        return (
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
        );
    }

    let ipv4_addr = nis[0].addr.iter().find(|a| a.ip().is_ipv4()).unwrap();
    (
        ipv4_addr.ip(),
        ipv4_addr
            .broadcast()
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))),
    )
}

pub async fn discovery(
    timeout: Duration,
) -> anyhow::Result<impl Stream<Item = anyhow::Result<(Daikin, DaikinInfo)>>> {
    let (srcip, dstip) = get_ipaddr();
    let src_addr = format!("{}:30000", srcip);
    let dst_addr = format!("{}:30050", dstip);

    debug!(
        "discovering daikin device from {} to {}",
        src_addr, dst_addr
    );

    let socket = UdpSocket::bind(src_addr).await?;
    socket.set_broadcast(true)?;
    let payload = "DAIKIN_UDP/common/basic_info";

    socket.send_to(&payload.as_bytes(), dst_addr).await?;

    Ok(gen!({
        loop {
            let mut buf = [0; 2048];
            let (text, src_addr) =
                match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                    Err(e) => {
                        warn!("stop discovering");
                        yield_!(Err(e.into()));
                        break;
                    }
                    Ok(res) => match res {
                        Ok((buf_size, _)) if buf_size == 2048 => {
                            warn!("UDP buffer too small");
                            continue;
                        }
                        Ok((buf_size, SocketAddr::V4(src_addr))) => {
                            match str::from_utf8(&buf[..buf_size]) {
                                Ok(val) => (val, src_addr),
                                Err(e) => {
                                    yield_!(Err(e.into()));
                                    continue;
                                }
                            }
                        }
                        Ok(_) => {
                            continue;
                        }
                        Err(e) => {
                            yield_!(Err(e.into()));
                            continue;
                        }
                    },
                };

            let item: HashMap<String, String> =
                form_urlencoded::parse(text.replace(",", "&").as_bytes())
                    .into_owned()
                    .collect();

            let daikin = Daikin::new(src_addr.ip().clone());
            let info = DaikinInfo::new(
                item.get("name").cloned(),
                item.get("mac").cloned(),
                item.get("ver").cloned(),
                item.get("edid").cloned(),
            );

            info!(
                "found daikin device at {}: {}",
                src_addr.ip(),
                info.name().unwrap_or("Unknown name".into())
            );

            yield_!(Ok((daikin, info)));
        }
    }))
}
