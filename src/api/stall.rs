use std::collections::HashMap;

use super::{Response, WeLoveClient};
use crate::error::{WeLoveError, WeLoveResult};
use rand::distributions::DistString;
use serde::{Deserialize, Serialize};

pub trait StallApi {
    async fn stall_query(&self) -> WeLoveResult<StallInfo>;
    async fn stall_earn(&self, slot: i64, stall_sale_id: i64) -> WeLoveResult<Response>;
    async fn stall_buy(&self, stall_sale_id: i64, seller_farm_id: i64) -> WeLoveResult<Response>;
    async fn stall_onshelf(
        &self,
        slot: i64,
        item_id: i64,
        count: i64,
        coin: i64,
        ad: bool,
        rainbow_coin: i64,
    ) -> WeLoveResult<Response>;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StallInfo {
    pub last_free_ad_time: i64,
    pub op_time: i64,
    pub msg_type: i64,
    pub stall_items: Vec<StallItem>,
    pub ad_auth: i64,
    pub capacity: i64,
    pub farm_id: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StallItem {
    pub buyer_head_url: String,
    pub item_id: i64,
    pub count: i64,
    pub buyer_farm_name: String,
    pub last_ad_time: i64,
    pub id: i64,
    pub slot: i64,
    pub buyer_lover_head_url: String,
    pub status: i64,
    pub coin: i64,
}

impl StallApi for WeLoveClient {
    async fn stall_query(&self) -> WeLoveResult<StallInfo> {
        serde_json::from_value(
            self.post("/v1/game/farm/stall/query", Default::default())
                .await?
                .messages
                .into_iter()
                .find(|m| m["msg_type"] == 20)
                .ok_or(WeLoveError::None("failed to get message msg_type=20"))?,
        )
        .map_err(WeLoveError::from)
    }

    async fn stall_earn(&self, slot: i64, stall_sale_id: i64) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/stall/earn",
            HashMap::from([
                ("slot", slot.to_string().as_str()),
                ("stall_sale_id", stall_sale_id.to_string().as_str()),
            ]),
        )
        .await
    }

    async fn stall_buy(&self, stall_sale_id: i64, seller_farm_id: i64) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/stall/buy",
            HashMap::from([
                ("stall_sale_id", stall_sale_id.to_string().as_str()),
                ("seller_farm_id", seller_farm_id.to_string().as_str()),
            ]),
        )
        .await
    }

    async fn stall_onshelf(
        &self,
        slot: i64,
        item_id: i64,
        count: i64,
        coin: i64,
        ad: bool,
        rainbow_coin: i64,
    ) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/stall/onshelf",
            HashMap::from([
                ("slot", slot.to_string().as_str()),
                ("item_id", item_id.to_string().as_str()),
                ("count", count.to_string().as_str()),
                ("coin", coin.to_string().as_str()),
                ("ad", (ad as u32).to_string().as_str()),
                ("rainbow_coin", rainbow_coin.to_string().as_str()),
                (
                    "check",
                    rand::distributions::Alphanumeric
                        .sample_string(&mut rand::thread_rng(), 6)
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
    async fn test_stall_query() {
        let cli = get_test_client();
        let items = cli.stall_query().await.unwrap();
        dbg!(items);
    }

    #[tokio::test]
    async fn test_stall_earn() {
        let cli = get_test_client();
        let resp = cli.stall_earn(3, 3226224553).await.unwrap();
        dbg!(resp);
    }

    #[tokio::test]
    async fn test_stall_buy() {
        let cli = get_test_client();
        let resp = cli.stall_buy(3226224553, 3226224553).await.unwrap();
        dbg!(resp);
    }

    #[tokio::test]
    async fn test_stall_onshelf() {
        let cli = get_test_client();
        let resp = cli
            .stall_onshelf(3, 201001, 10, 36, false, 0)
            .await
            .unwrap();
        dbg!(resp);
    }

    #[tokio::test]
    async fn test_stall_earn_all() {
        let cli = get_test_client();
        let stall = cli.stall_query().await.unwrap();
        for item in stall
            .stall_items
            .into_iter()
            .filter(|item| item.status == 2)
        {
            let resp = cli.stall_earn(item.slot, item.id).await;
            println!("{resp:?}");
        }
    }
}
