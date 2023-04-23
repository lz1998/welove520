use crate::error::{WeLoveError, WeLoveResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Response, WeLoveClient};
pub trait OrderApi {
    async fn order_query(&self) -> WeLoveResult<OrderInfo>;
    async fn order_refuse(&self, order_id: i64) -> WeLoveResult<Response>;
    async fn order_accomplish(
        &self,
        order_id: i64,
        by_rainbow_coin: bool,
    ) -> WeLoveResult<Response>;
    async fn order_reward(&self, order_id: i64) -> WeLoveResult<Response>;
}
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderInfo {
    pub op_time: i64,
    pub msg_type: i64,
    pub orders: Vec<Order>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Order {
    pub crystal_item_id: i64,
    #[serde(rename = "descId")]
    pub desc_id: i64,
    pub voucher_item_id: i64,
    pub icon: i64,
    pub op_time: i64,
    pub slot: i64,
    pub buyer: i64,
    pub special: i64,
    pub time_left: i64,
    pub exp: i64,
    pub items: Vec<OrderItem>,
    pub order_id: i64,
    pub status: i64,
    pub coin: i64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderItem {
    pub item_id: i64,
    pub count: i64,
}

impl OrderApi for WeLoveClient {
    async fn order_query(&self) -> WeLoveResult<OrderInfo> {
        serde_json::from_value(
            self.post("/v1/game/farm/order/query", Default::default())
                .await?
                .messages
                .into_iter()
                .find(|m| m["msg_type"] == 15)
                .ok_or(WeLoveError::None("failed to get message msg_type=15"))?,
        )
        .map_err(WeLoveError::from)
    }

    async fn order_refuse(&self, order_id: i64) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/order/refuse",
            HashMap::from([("order_id", order_id.to_string().as_str())]),
        )
        .await
    }

    async fn order_accomplish(
        &self,
        order_id: i64,
        by_rainbow_coin: bool,
    ) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/order/accomplish",
            HashMap::from([
                ("order_id", order_id.to_string().as_str()),
                (
                    "by_rainbow_coin",
                    (by_rainbow_coin as i32).to_string().as_str(),
                ),
            ]),
        )
        .await
    }

    async fn order_reward(&self, order_id: i64) -> WeLoveResult<Response> {
        self.post(
            "/v1/game/farm/order/reward",
            HashMap::from([("order_id", order_id.to_string().as_str())]),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::get_test_client;

    #[tokio::test]
    async fn test_order_query() {
        let cli = get_test_client();
        let resp = cli.order_query().await.unwrap();
        dbg!(resp);
    }

    #[tokio::test]
    async fn test_order_refuse() {
        let cli = get_test_client();
        let resp = cli.order_query().await.unwrap();
        for order in resp
            .orders
            .into_iter()
            .filter(|o| o.special != 1 && o.time_left == -1)
        {
            let resp = cli.order_refuse(order.order_id).await;
            let _ = dbg!(resp);
        }
    }

    #[tokio::test]
    async fn test_order_accomplish() {
        let cli = get_test_client();
        // 209002
        let resp = cli.order_accomplish(2434433066, false).await.unwrap();

        println!("{}", serde_json::to_string(&resp).unwrap());
        if let Some(order) = resp.messages.into_iter().find(|m| m["msg_type"] == 47) {
            let new_order: Result<Order, _> = serde_json::from_value(order);
            println!("{:?}", new_order);
        }
        // dbg!(resp);
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let _ = cli.order_reward(2434433066).await.unwrap();
    }

    #[tokio::test]
    async fn test_order_reward() {
        let cli = get_test_client();
        // 209002
        let resp = cli.order_reward(2434423817).await.unwrap();
        dbg!(resp);
    }
}
