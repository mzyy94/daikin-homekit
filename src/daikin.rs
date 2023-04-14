use crate::discovery;
use crate::error::Error;
use crate::info::DaikinInfo;
use crate::response::{DaikinResponse, Response};
use crate::status::DaikinStatus;
use futures::prelude::*;
use retainer::*;
use serde_json::json;
use serde_json::value::Value;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Daikin {
    endpoint: String,
    cache: Arc<Cache<u8, DaikinStatus>>,
}

impl Daikin {
    pub fn new(ip_addr: Ipv4Addr) -> Daikin {
        Daikin {
            endpoint: format!("http://{}/dsiot/multireq", ip_addr),
            cache: Arc::new(Cache::new()),
        }
    }

    pub async fn discovery(timeout: Duration) -> Option<(Daikin, DaikinInfo)> {
        if let Ok(mut stream) = discovery::discovery(timeout).await {
            if let Some(item) = stream.next().await {
                match item {
                    Ok(item) => Some(item),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn send_request(&self, payload: Value) -> Result<String, Error> {
        let client = reqwest::Client::builder()
            .http1_title_case_headers()
            .build()?;

        let resp = client.post(&self.endpoint).json(&payload).send().await?;

        if resp.status() != reqwest::StatusCode::OK {
            dbg!(resp.status());
            return Err(Error::HTTPError(resp.status()));
        }

        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn get_status(&self) -> Result<DaikinStatus, Error> {
        let status: DaikinStatus = match self.cache.get(&1).await {
            Some(cache) => *cache.value(),
            None => {
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
                let res: DaikinResponse = serde_json::from_str(&body)?;
                let rsc_error: Vec<Response> = res
                    .responses
                    .iter()
                    .filter(|r| r.rsc / 10 != 200)
                    .map(|r| r.to_owned())
                    .collect();
                if rsc_error.len() > 0 {
                    return Err(Error::RSCError(rsc_error));
                }

                let status = DaikinStatus::new(res);

                self.cache.insert(1, status, 5000).await;
                status
            }
        };

        Ok(status)
    }

    pub async fn get_info(&self) -> Result<DaikinInfo, Error> {
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

    pub async fn update(&self, status: DaikinStatus) -> Result<(), Error> {
        let request = status.build_request();
        let payload = serde_json::to_value(request)?;
        let _ = self.send_request(payload).await?;
        self.cache.insert(1, status, 3000).await;

        Ok(())
    }
}

impl std::fmt::Debug for Daikin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Daikin")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}
