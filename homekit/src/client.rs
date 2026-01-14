use std::time::Duration;

use dsiot::daikin::HttpClient;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn try_new() -> Result<Self, reqwest::Error> {
        Ok(ReqwestClient {
            client: reqwest::Client::builder()
                .http1_title_case_headers()
                .timeout(Duration::new(5, 0))
                .build()?,
        })
    }
}

impl HttpClient for ReqwestClient {
    async fn send_request(&self, url: String, payload: Value) -> anyhow::Result<Value> {
        let response = self.client.post(&url).json(&payload).send().await?;
        let body = response.json().await?;
        Ok(body)
    }
}
