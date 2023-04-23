pub mod crops;
pub mod market;
pub mod order;
pub mod panorama;
pub mod stall;

use crate::error::{WeLoveError, WeLoveResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Response {
    pub result: u32,
    pub messages: Vec<serde_json::Value>,
    pub error_msg: String,
}

pub struct WeLoveClient {
    pub http_client: reqwest::Client,
    pub base_url: String,
    pub default_params: HashMap<String, String>,
}

impl WeLoveClient {
    pub fn new(
        base_url: String,
        default_params: HashMap<String, String>,
        default_headers: HashMap<String, String>,
    ) -> Self {
        Self {
            http_client: reqwest::ClientBuilder::new()
                .default_headers((&default_headers).try_into().unwrap())
                .build()
                .unwrap(),
            base_url,
            default_params,
        }
    }

    pub async fn post(
        &self,
        path: &str,
        mut params: HashMap<&str, &str>,
    ) -> WeLoveResult<Response> {
        for (k, v) in self.default_params.iter() {
            params.insert(k, v);
        }
        let timestamp = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap()
            .as_millis()
            .to_string();
        params.insert("ts", &timestamp);
        let sig = crate::utils::sig::calculate_sig("POST", path, &params);
        params.insert("sig", &sig);
        self.http_client
            .post(format!("{}{path}", self.base_url))
            .form(&params)
            .send()
            .await?
            .json()
            .await
            .map_err(WeLoveError::from)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::api::WeLoveClient;
    use std::collections::HashMap;

    pub fn get_test_client() -> WeLoveClient {
        let base_url = std::env::var("BASE_URL").expect("env BASE_URL is not set");
        let version = std::env::var("VERSION").expect("env VERSION is not set");
        let union_id = std::env::var("UNION_ID").expect("env UNION_ID is not set");

        WeLoveClient::new(
            base_url,
            HashMap::from([
                ("fv".to_string(), version),
                ("union_id".to_string(), union_id),
            ]),
            Default::default(),
        )
    }
}
