use std::collections::HashMap;

use super::{Response, WeLoveClient};
use crate::error::WeLoveResult;
use serde::{Deserialize, Serialize};

pub trait CropsApi {
    async fn crops_plant(
        &self,
        item_id: i64,
        farmland_ids: Vec<Farmland>,
    ) -> WeLoveResult<Response>;
    async fn crops_harvest(&self, item_id: i64, farmland_ids: Vec<i64>) -> WeLoveResult<Response>;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Farmland {
    pub id: i64,
    pub last_interval: i64,
    pub x: i64,
    pub y: i64,
}

impl CropsApi for WeLoveClient {
    async fn crops_plant(&self, item_id: i64, farmlands: Vec<Farmland>) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/crops/plant",
            HashMap::from([
                ("item_id", item_id.to_string().as_str()),
                (
                    "farmlands",
                    serde_json::to_string(&farmlands).unwrap().as_str(),
                ),
            ]),
        )
        .await
    }

    async fn crops_harvest(&self, item_id: i64, farmland_ids: Vec<i64>) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/crops/harvest",
            HashMap::from([
                ("item_id", item_id.to_string().as_str()),
                (
                    "farmland_ids",
                    farmland_ids
                        .into_iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                        .as_str(),
                ),
            ]),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::get_test_client;

    #[tokio::test]
    async fn test_crops_plant() {
        let cli = get_test_client();
        let items = cli.crops_plant(201001, vec![]).await.unwrap();
        dbg!(items);
    }
}
