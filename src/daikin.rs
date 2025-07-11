use dsiot::info::DaikinInfo;
use dsiot::request::DaikinRequest;
use dsiot::response::DaikinResponse;
use dsiot::status::DaikinStatus;
use retainer::*;
use serde_json::json;
use serde_json::value::Value;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[allow(async_fn_in_trait)]
pub trait HttpClient {
    async fn send_request(&self, url: String, payload: Value) -> anyhow::Result<Value>;
}

#[derive(Clone)]
pub struct Daikin<H: HttpClient> {
    endpoint: String,
    cache: Arc<Cache<u8, DaikinStatus>>,
    client: Arc<H>,
}

impl<H: HttpClient> std::fmt::Debug for Daikin<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Daikin {{ endpoint: {} }}", self.endpoint,)
    }
}

impl<H: HttpClient> Daikin<H> {
    pub fn new(ip_addr: Ipv4Addr, client: H) -> Daikin<H> {
        Daikin {
            endpoint: format!("http://{ip_addr}/dsiot/multireq"),
            cache: Arc::new(Cache::new()),
            client: Arc::new(client),
        }
    }

    pub async fn get_status(&self) -> anyhow::Result<DaikinStatus> {
        let status: DaikinStatus = match self.cache.get(&1).await {
            Some(cache) => cache.value().clone(),
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

                let body = self
                    .client
                    .send_request(self.endpoint.clone(), payload)
                    .await?;
                let status: DaikinStatus = serde_json::from_value::<DaikinResponse>(body)?.into();

                self.cache.insert(1, status.clone(), 5000).await;
                status
            }
        };

        Ok(status)
    }

    pub async fn get_info(&self) -> anyhow::Result<DaikinInfo> {
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

        let body = self
            .client
            .send_request(self.endpoint.clone(), payload)
            .await?;
        let info: DaikinInfo = serde_json::from_value::<DaikinResponse>(body)?.into();

        Ok(info)
    }

    pub async fn update(&self, status: DaikinStatus) -> anyhow::Result<()> {
        let request: DaikinRequest = status.clone().into();
        let payload = serde_json::to_value(request)?;
        let _ = self
            .client
            .send_request(self.endpoint.clone(), payload)
            .await?;
        self.cache.insert(1, status, 3000).await;

        Ok(())
    }
}
