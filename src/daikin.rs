use crate::status::DaikinStatus;
use serde_json::json;

pub struct Daikin {
    endpoint: String,
}

impl Daikin {
    pub fn new(ip_addr: String) -> Daikin {
        Daikin {
            endpoint: format!("http://{}/dsiot/multireq", ip_addr),
        }
    }

    pub async fn get_status(&self) -> Result<DaikinStatus, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .http1_title_case_headers()
            .build()?;
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

        let resp = client.post(&self.endpoint).json(&payload).send().await?;

        if resp.status() != reqwest::StatusCode::OK {
            dbg!(resp.status());
            todo!();
        }

        let body = resp.text().await?;
        let res: DaikinStatus = serde_json::from_str(&body)?;

        Ok(res)
    }
}
