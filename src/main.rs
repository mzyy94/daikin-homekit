mod status;

use crate::status::DaikinStatus;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1) {
        Some(ip_addr) => get_status(ip_addr).await,
        None => Ok(()),
    }
}

async fn get_status(ip_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .http1_title_case_headers()
        .build()?;
    let url = format!("http://{}/dsiot/multireq", ip_addr);
    let payload = json!({"requests": [
        {
            "op": 2,
            "to": "/dsiot/edge/adr_0100.dgc_status?filter=pv,md"
        },
        {
            "op": 2,
            "to": "/dsiot/edge/adr_0200.dgc_status?filter=pv,md"
        }
    ]});

    let resp = client.post(url).json(&payload).send().await?;

    if resp.status() != reqwest::StatusCode::OK {
        println!("{}", resp.status());
        return Ok(());
    }

    let body = resp.text().await?;
    let res: DaikinStatus = serde_json::from_str(&body)?;

    println!("{:#?}", res);

    let val = res.get("/dsiot/edge/adr_0100.dgc_status", "e_1002/e_A00B/p_01");
    println!("current temperature: {}", val.unwrap().get_f64().unwrap());

    Ok(())
}
