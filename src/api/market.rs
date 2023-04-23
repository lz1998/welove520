use crate::error::{WeLoveError, WeLoveResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Response, WeLoveClient};
pub trait MarketApi {
    async fn market_sale(&self, item_id: i64, count: i64) -> WeLoveResult<Response>;
    async fn market_query(&self) -> WeLoveResult<MarketInfo>;
    async fn market_buy(&self, id: i64) -> WeLoveResult<Response>;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarketInfo {
    pub op_time: i64,
    pub msg_type: i64,
    pub market_item_list: Vec<MarketItem>,
    pub next_refresh_time: i64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarketItem {
    pub item_id: i64,
    pub count: i64,
    pub id: i64,
    #[serde(rename = "soldOut")]
    pub sold_out: i64,
    pub coin: i64,
}

impl MarketApi for WeLoveClient {
    async fn market_sale(&self, item_id: i64, count: i64) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/market/sale",
            HashMap::from([
                ("item_id", item_id.to_string().as_str()),
                ("count", count.to_string().as_str()),
            ]),
        )
        .await
    }

    async fn market_query(&self) -> WeLoveResult<MarketInfo> {
        serde_json::from_value(
            self.post("/v1/game/farm/market/query", Default::default())
                .await?
                .messages
                .into_iter()
                .find(|m| m["msg_type"] == 920)
                .ok_or(WeLoveError::None("failed to get message msg_type=920"))?,
        )
        .map_err(WeLoveError::from)
    }

    async fn market_buy(&self, id: i64) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/market/buy",
            HashMap::from([("id", id.to_string().as_str())]),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::get_test_client;

    #[tokio::test]
    async fn test_market_sale() {
        let cli = get_test_client();
        let resp = cli.market_sale(201001, 10).await.unwrap();
        dbg!(resp);
    }

    #[tokio::test]
    async fn test_market_query() {
        let cli = get_test_client();
        let market = cli.market_query().await.unwrap();
        dbg!(market);
    }

    #[tokio::test]
    async fn test_market_buy() {
        let cli = get_test_client();
        let resp = cli.market_buy(3).await.unwrap();
        dbg!(resp);
    }
}
