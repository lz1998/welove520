use std::cmp::Ordering;
use std::collections::HashMap;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use welove520::api::crops::{CropsApi, Farmland};
use welove520::api::market::MarketApi;
use welove520::api::order::OrderApi;
use welove520::api::panorama::PanoramaApi;
use welove520::api::stall::StallApi;
use welove520::api::WeLoveClient;

const WHEAT_ITEM_ID: i64 = 201001;
const BUY_IDS: [i64; 8] = [209001, 209002, 209003, 209004, 210001, 210002, 210003, 210004];

#[tokio::main]
async fn main() {
    init_log();
    let base_url = std::env::var("BASE_URL").expect("env BASE_URL is not set");
    let version = std::env::var("VERSION").expect("env VERSION is not set");
    let union_id = std::env::var("UNION_ID").expect("env UNION_ID is not set");
    let cli = WeLoveClient::new(
        base_url,
        HashMap::from([
            ("fv".to_string(), version),
            ("union_id".to_string(), union_id),
        ]),
        Default::default(),
    );
    let mut i = 0;
    loop {
        tracing::info!("loop: {i}");
        i += 1;
        harvest_and_plant(&cli).await;
        let harvest_sleep = tokio::time::sleep(std::time::Duration::from_secs(125));
        let mut warehouse_items = get_warehouse_items(&cli).await;
        tracing::info!(
            "after harvest_and_plant, wheat_count: {}",
            get_warehouse_item_count(&warehouse_items, WHEAT_ITEM_ID)
        );
        stall_renew(&cli, &mut warehouse_items).await;
        tracing::info!(
            "after stall_renew, wheat_count: {}",
            get_warehouse_item_count(&warehouse_items, WHEAT_ITEM_ID)
        );
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        sale_remain_wheat(&cli, &mut warehouse_items, 10).await;
        auto_orders(&cli, &mut warehouse_items).await;
        buy_ingot(&cli).await;
        harvest_sleep.await;
    }
}

async fn harvest_and_plant(cli: &WeLoveClient) {
    let fields: Vec<_> = cli.get_fields().await.unwrap_or_default();

    let mut harvest_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.plant_item_id == WHEAT_ITEM_ID && f.left_time < 0) // TODO harvest all？
        .collect();
    harvest_fields.sort_unstable_by(|a, b| match a.x.cmp(&b.x) {
        Ordering::Equal => a.y.cmp(&b.y),
        ord => ord,
    });

    let mut empty_fields: Vec<_> = fields.iter().filter(|f| f.plant_item_id == -1).collect();

    let harvest_farmland_ids = harvest_fields.iter().map(|f| f.id).collect();
    tracing::info!("harvest_farmland_ids: {harvest_farmland_ids:?}");
    if let Err(err) = cli.crops_harvest(201001, harvest_farmland_ids).await {
        tracing::error!("failed to harvest: {err}");
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    empty_fields.extend(harvest_fields);
    let mut farmlands: Vec<_> = empty_fields
        .into_iter()
        .map(|f| Farmland {
            id: f.id,
            last_interval: 1,
            x: f.x,
            y: f.y,
        })
        .collect();
    farmlands.sort_unstable_by(|a, b| match a.x.cmp(&b.x) {
        Ordering::Equal => a.y.cmp(&b.y),
        ord => ord,
    });
    if !farmlands.is_empty() {
        tracing::info!(
            "plant_farmlands: {:?}",
            farmlands.iter().map(|f| f.id).collect::<Vec<_>>()
        );
        if let Err(err) = cli.crops_plant(WHEAT_ITEM_ID, farmlands).await {
            tracing::error!("failed to plant: {err}");
        }
    }
}

async fn stall_renew(cli: &WeLoveClient, warehouse_items: &mut HashMap<i64, i64>) {
    let stall = match cli.stall_query().await {
        Ok(stall) => stall,
        Err(err) => {
            tracing::error!("failed to query stall: {err}");
            return;
        }
    };
    tracing::info!("stall last_free_ad_time: {}", stall.last_free_ad_time);
    let mut ad = stall.last_free_ad_time == 0;
    let not_empty_slots: Vec<_> = stall.stall_items.iter().map(|item| item.slot).collect();
    let empty_slots: Vec<_> = (1..=stall.capacity)
        .filter(|slot| !not_empty_slots.contains(slot))
        .collect();
    tracing::info!("stall empty_slots: {empty_slots:?}");
    for slot in empty_slots {
        if get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) >= 10 {
            if let Err(err) = cli
                .stall_onshelf(slot, WHEAT_ITEM_ID, 10, 36, std::mem::take(&mut ad), 0)
                .await
            {
                tracing::error!("failed to onshelf: {err}");
            } else {
                tracing::info!("succeed to onshelf, slot: {}", slot);
                warehouse_items.insert(
                    WHEAT_ITEM_ID,
                    get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) - 10,
                );
            }
        }
    }

    for item in stall.stall_items.iter().filter(|item| item.status == 2) {
        if let Err(err) = cli.stall_earn(item.slot, item.id).await {
            tracing::error!("failed to earn: {err}");
        } else if get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) >= 10 {
            if let Err(err) = cli
                .stall_onshelf(item.slot, WHEAT_ITEM_ID, 10, 36, std::mem::take(&mut ad), 0)
                .await
            {
                tracing::error!("failed to onshelf: {err}");
            } else {
                tracing::info!("succeed to onshelf after earn, slot: {}", item.slot);
                warehouse_items.insert(
                    WHEAT_ITEM_ID,
                    get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) - 10,
                );
            }
        } else {
            tracing::info!(
                "succeed to earn, slot: {}, wheat is not enough: {}",
                item.slot,
                get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID)
            );
        }
    }
}

async fn get_warehouse_items(cli: &WeLoveClient) -> HashMap<i64, i64> {
    cli.get_warehouses()
        .await
        .unwrap_or_default()
        .into_iter()
        .flat_map(|w| w.items)
        .map(|item| (item.item_id, item.count))
        .collect()
}

fn get_warehouse_item_count(warehouse_items: &HashMap<i64, i64>, item_id: i64) -> i64 {
    warehouse_items.get(&item_id).copied().unwrap_or_default()
}

async fn sale_remain_wheat(
    cli: &WeLoveClient,
    warehouse_items: &mut HashMap<i64, i64>,
    remain: i64,
) {
    if get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) > remain {
        if let Err(err) = cli
            .market_sale(
                WHEAT_ITEM_ID,
                get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) - remain,
            )
            .await
        {
            tracing::error!("failed to sale wheat: {err}")
        } else {
            tracing::info!(
                "after sale remain wheat, count: {}, remain: 10",
                get_warehouse_item_count(warehouse_items, WHEAT_ITEM_ID) - remain
            );
            warehouse_items.insert(WHEAT_ITEM_ID, 10);
        }
    }
}

async fn auto_orders(cli: &WeLoveClient, warehouse_items: &mut HashMap<i64, i64>) {
    let order_info = match cli.order_query().await {
        Ok(order_info) => order_info,
        Err(err) => {
            tracing::error!("failed to query order: {err}");
            return;
        }
    };
    let (orders, mut waiting_slots): (Vec<_>, Vec<_>) = order_info
        .orders
        .into_iter()
        .partition(|o| o.time_left <= 0);

    if !waiting_slots.is_empty() {
        waiting_slots.sort_unstable_by_key(|s| s.time_left);
        tracing::info!(
            "waiting order time: {:?}",
            waiting_slots
                .iter()
                .map(|s| std::time::Duration::from_secs(s.time_left as u64))
                .collect::<Vec<_>>()
        );
    }
    if !orders.is_empty() {
        tracing::info!("ready order count: {}", orders.len());
    }

    for mut order in orders.into_iter() {
        loop {
            let order_item_count: i64 = order.items.iter().map(|item| item.count).sum();
            if !order
                .items
                .iter()
                .all(|item| get_warehouse_item_count(warehouse_items, item.item_id) >= item.count)
                || order_item_count > 2
            {
                if order.voucher_item_id == 0 || order_item_count > 2 {
                    if let Err(err) = cli.order_refuse(order.order_id).await {
                        tracing::error!("failed to refuse order: {err}")
                    } else {
                        tracing::info!(
                            "succeed to refuse order, special: {}, slot: {}, order_id: {}, items: {}",
                            order.special,
                            order.slot,
                            order.order_id,
                            serde_json::to_string(&order.items).unwrap()
                        )
                    }
                } else {
                    tracing::info!(
                        "special order, item is not enough, slot: {} ,order_id: {}",
                        order.slot,
                        order.order_id
                    );
                }
                break;
            }
            let mut accomplish_resp = match cli.order_accomplish(order.order_id, false).await {
                Ok(resp) => resp,
                Err(err) => {
                    tracing::error!("failed to accomplish order: {err}");
                    break;
                }
            };
            tracing::info!(
                "succeed to accomplish order, special: {}, slot: {}, order_id: {}, order: {}",
                order.special,
                order.slot,
                order.order_id,
                serde_json::to_string(&order).unwrap()
            );
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            if let Err(err) = cli.order_reward(order.order_id).await {
                tracing::error!("failed to reward order: {err}");
            }
            for item in order.items.iter() {
                warehouse_items.insert(
                    item.item_id,
                    get_warehouse_item_count(warehouse_items, item.item_id) - item.count,
                );
            }
            if let Some(msg) = accomplish_resp
                .messages
                .iter_mut()
                .find(|m| m["msg_type"] == 47)
            {
                order = serde_json::from_value(msg.take()).unwrap();
            } else {
                tracing::error!(
                    "error accomplish msg: {}",
                    serde_json::to_string(&accomplish_resp).unwrap()
                );
            }
        }
    }
}
async fn buy_ingot(cli: &WeLoveClient) {
    let market_info = match cli.market_query().await {
        Ok(market_info) => market_info,
        Err(err) => {
            tracing::error!("failed to markket_query: {err}");
            return;
        }
    };
    for item in market_info
        .market_item_list
        .iter()
        .filter(|item| BUY_IDS.contains(&item.item_id) && item.sold_out == 0)
    {
        if let Err(err) = cli.market_buy(item.id).await {
            tracing::error!("failed to market_buy: {err}")
        } else {
            tracing::info!(
                "succeed to market_buy, item_id: {}, count: {}",
                item.item_id,
                item.count
            )
        }
    }
}

fn init_log() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
                    time::UtcOffset::__from_hms_unchecked(8, 0, 0),
                    time::macros::format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second]"
                    ),
                )),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("main", Level::DEBUG)
                .with_target("welove520", Level::DEBUG),
        )
        .init();
}
