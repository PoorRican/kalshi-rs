use crate::types::{
    BookSide, FixedPointCount, FixedPointDollars, OrderStatus, SelfTradePreventionType, YesNo,
};
use serde::Deserialize;

/// User order update payload (type: "user_order").
#[derive(Debug, Clone, Deserialize)]
pub struct WsUserOrder {
    pub order_id: String,
    pub user_id: String,
    pub ticker: String,
    /// Exchange shard the order resides on. Added 2026-08-27.
    #[serde(default)]
    pub exchange_index: Option<i64>,
    #[serde(default)]
    pub status: Option<OrderStatus>,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `outcome_side`.
    #[serde(default)]
    pub side: Option<YesNo>,
    #[serde(default)]
    pub is_yes: Option<bool>,
    /// Normalized outcome side (yes | no). Added 2026-05-07.
    #[serde(default)]
    pub outcome_side: Option<YesNo>,
    /// Normalized book side (bid | ask). Added 2026-05-07.
    #[serde(default)]
    pub book_side: Option<BookSide>,
    #[serde(default)]
    pub yes_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub fill_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub remaining_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub initial_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub taker_fill_cost_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub maker_fill_cost_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub taker_fees_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub maker_fees_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub created_ts_ms: Option<i64>,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub last_updated_ts_ms: Option<i64>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub expiration_ts_ms: Option<i64>,
    #[serde(default)]
    pub subaccount_number: Option<u32>,
}
