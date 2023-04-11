use crate::info::DaikinInfo;
use crate::status::DaikinStatus;
use serde_json::json;
use serde_json::value::Value;
use std::net::Ipv4Addr;

pub struct Daikin {
    endpoint: String,
}

impl Daikin {
    pub fn new(ip_addr: Ipv4Addr) -> Daikin {
        Daikin {
            endpoint: format!("http://{}/dsiot/multireq", ip_addr),
        }
    }

    async fn send_request(&self, payload: Value) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .http1_title_case_headers()
            .build()?;

        let resp = client.post(&self.endpoint).json(&payload).send().await?;

        if resp.status() != reqwest::StatusCode::OK {
            dbg!(resp.status());
            todo!();
        }

        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn get_status(&self) -> Result<DaikinStatus, Box<dyn std::error::Error>> {
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

        let body = self.send_request(payload).await?;
        let res: DaikinStatus = serde_json::from_str(&body)?;

        Ok(res)
    }

    pub async fn get_info(&self) -> Result<DaikinInfo, Box<dyn std::error::Error>> {
        let payload = json!({"requests": [
            {
                "op": 2,
                "to": "/dsiot/edge.adp_i"
            },
            {
                "op": 2,
                "to": "/dsiot/edge.adp_d"
            }
        ]});

        let body = self.send_request(payload).await?;
        let res: DaikinInfo = serde_json::from_str(&body)?;

        Ok(res)
    }

    pub async fn update(&self, status: DaikinStatus) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_value(status).unwrap();
        let _ = self.send_request(payload).await?;

        Ok(())
    }
}
