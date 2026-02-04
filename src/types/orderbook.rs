use serde::{Deserialize, Serialize};

use super::FixedPointDollars;

#[derive(Debug, Clone, Deserialize)]
pub struct Orderbook {
    /// Price levels: (price_cents, quantity)
    #[serde(default)]
    pub yes: Vec<(i64, i64)>,
    /// Price levels: (price_cents, quantity)
    #[serde(default)]
    pub no: Vec<(i64, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default)]
    pub yes_dollars: Vec<(FixedPointDollars, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default)]
    pub no_dollars: Vec<(FixedPointDollars, i64)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookFp {
    /// Price levels: (price_dollars, quantity_fp)
    #[serde(default)]
    pub yes_dollars: Vec<(FixedPointDollars, String)>,
    /// Price levels: (price_dollars, quantity_fp)
    #[serde(default)]
    pub no_dollars: Vec<(FixedPointDollars, String)>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GetMarketOrderbookParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketOrderbookResponse {
    pub orderbook: Orderbook,
    #[serde(default)]
    pub orderbook_fp: Option<OrderbookFp>,
}
