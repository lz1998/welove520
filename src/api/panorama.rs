use crate::error::{WeLoveError, WeLoveResult};
use serde::{Deserialize, Serialize};

use super::{Response, WeLoveClient};

pub trait PanoramaApi {
    async fn panorama(&self) -> WeLoveResult<Response>;
    async fn get_fields(&self) -> WeLoveResult<Vec<Field>>;
    async fn get_warehouses(&self) -> WeLoveResult<Vec<Warehouse>>;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Field {
    pub rotate: i64,
    pub plant_time: i64,
    pub item_id: i64,
    pub product_status: i64,
    pub left_time: i64,
    pub x: i64,
    pub y: i64,
    pub time: i64,
    pub id: i64,
    pub plant_item_id: i64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Warehouse {
    pub category: i64,
    pub items: Vec<ItemInfo>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemInfo {
    pub count: i64,
    pub item_id: i64,
}

impl PanoramaApi for WeLoveClient {
    async fn panorama(&self) -> WeLoveResult<Response> {
        self.post("/v1/game/farm/panorama", Default::default())
            .await
    }

    async fn get_fields(&self) -> WeLoveResult<Vec<Field>> {
        serde_json::from_value(
            self.panorama()
                .await?
                .messages
                .into_iter()
                .find(|m| m["msg_type"] == 2)
                .ok_or(WeLoveError::None("failed to get message msg_type=2"))?["fields"]
                .take(),
        )
        .map_err(WeLoveError::from)
    }

    async fn get_warehouses(&self) -> WeLoveResult<Vec<Warehouse>> {
        serde_json::from_value(
            self.panorama()
                .await?
                .messages
                .into_iter()
                .find(|m| m["msg_type"] == 3)
                .ok_or(WeLoveError::None("failed to get message msg_type=3"))?["warehouses"]
                .take(),
        )
        .map_err(WeLoveError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::get_test_client;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_panorama() {
        let cli = get_test_client();
        let resp = cli.panorama().await.unwrap();
        let resp = resp
            .messages
            .into_iter()
            .find(|m| m["msg_type"] == 2)
            .unwrap();
        println!("{:?}", resp["fields"]);
    }

    #[tokio::test]
    async fn test_get_fields() {
        let cli = get_test_client();
        let resp = cli.get_fields().await.unwrap();
        println!("{resp:?}");
    }

    #[tokio::test]
    async fn test_get_warehouses() {
        let cli = get_test_client();
        let resp = cli.get_warehouses().await.unwrap();
        println!("{resp:?}");
        let items: HashMap<_, _> = resp
            .into_iter()
            .map(|w| w.items)
            .flatten()
            .map(|item| (item.item_id, item.count))
            .collect();
        println!("{:?}", items)
    }
}
