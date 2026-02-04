use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

use crate::error::KalshiError;
use super::{FixedPointCount, FixedPointDollars, serialize_csv_opt};

#[derive(Debug, Clone, Deserialize)]
pub struct GetBalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
    pub updated_ts: i64,
}

/// count_filter accepted values: position, total_traded
#[derive(Debug, Clone, Copy)]
pub enum PositionCountFilter {
    Position,
    TotalTraded,
}

impl PositionCountFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            PositionCountFilter::Position => "position",
            PositionCountFilter::TotalTraded => "total_traded",
        }
    }
}
impl fmt::Display for PositionCountFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// GET /portfolio/positions query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetPositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000

    /// CSV of non-zero filters (position,total_traded)
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub count_filter: Option<Vec<PositionCountFilter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub event_ticker: Option<Vec<String>>,

    /// 0..=32
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

impl GetPositionsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 1000 {
                return Err(KalshiError::InvalidParams(
                    "GET /portfolio/positions: limit must be 1..=1000".to_string(),
                ));
            }
        }
        if let Some(evts) = &self.event_ticker {
            if evts.len() > 10 {
                return Err(KalshiError::InvalidParams(
                    "GET /portfolio/positions: event_ticker supports up to 10 tickers".to_string(),
                ));
            }
        }
        if let Some(sub) = self.subaccount {
            if sub > 32 {
                return Err(KalshiError::InvalidParams(
                    "subaccount must be 0..=32".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketPosition {
    pub ticker: String,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub position_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub fees_paid: Option<i64>,
    #[serde(default)]
    pub fees_paid_fp: Option<FixedPointDollars>,
    #[serde(default)]
    pub resting_orders: Option<i64>,
    #[serde(default)]
    pub resting_orders_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub total_traded: Option<i64>,
    #[serde(default)]
    pub total_traded_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventPosition {
    pub event_ticker: String,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub position_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub fees_paid: Option<i64>,
    #[serde(default)]
    pub fees_paid_fp: Option<FixedPointDollars>,
    #[serde(default)]
    pub resting_orders: Option<i64>,
    #[serde(default)]
    pub resting_orders_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub total_traded: Option<i64>,
    #[serde(default)]
    pub total_traded_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPositionsResponse {
    pub market_positions: Vec<MarketPosition>,
    pub event_positions: Vec<EventPosition>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Resting,
    Canceled,
    Executed,
    #[serde(other)]
    Unknown,
}

impl OrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderStatus::Resting => "resting",
            OrderStatus::Canceled => "canceled",
            OrderStatus::Executed => "executed",
            OrderStatus::Unknown => "unknown",
        }
    }
}
impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for OrderStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// GET /portfolio/orders query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub event_ticker: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 200

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

impl GetOrdersParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 200 {
                return Err(KalshiError::InvalidParams(
                    "GET /portfolio/orders: limit must be 1..=200".to_string(),
                ));
            }
        }
        if let Some(evts) = &self.event_ticker {
            if evts.len() > 10 {
                return Err(KalshiError::InvalidParams(
                    "GET /portfolio/orders: event_ticker supports up to 10 tickers".to_string(),
                ));
            }
        }
        if let Some(sub) = self.subaccount {
            if sub > 32 {
                return Err(KalshiError::InvalidParams(
                    "subaccount must be 0..=32".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YesNo {
    Yes,
    No,
    #[serde(other)]
    Unknown,
}
impl Default for YesNo {
    fn default() -> Self {
        YesNo::Yes
    }
}
impl YesNo {
    pub fn as_str(self) -> &'static str {
        match self {
            YesNo::Yes => "yes",
            YesNo::No => "no",
            YesNo::Unknown => "unknown",
        }
    }
}
impl fmt::Display for YesNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for YesNo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuySell {
    Buy,
    Sell,
    #[serde(other)]
    Unknown,
}
impl Default for BuySell {
    fn default() -> Self {
        BuySell::Buy
    }
}
impl BuySell {
    pub fn as_str(self) -> &'static str {
        match self {
            BuySell::Buy => "buy",
            BuySell::Sell => "sell",
            BuySell::Unknown => "unknown",
        }
    }
}
impl fmt::Display for BuySell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for BuySell {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Limit,
    Market,
    #[serde(other)]
    Unknown,
}
impl OrderType {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
            OrderType::Unknown => "unknown",
        }
    }
}
impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for OrderType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    FillOrKill,
    GoodTillCanceled,
    ImmediateOrCancel,
    #[serde(other)]
    Unknown,
}
impl TimeInForce {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeInForce::FillOrKill => "fill_or_kill",
            TimeInForce::GoodTillCanceled => "good_till_canceled",
            TimeInForce::ImmediateOrCancel => "immediate_or_cancel",
            TimeInForce::Unknown => "unknown",
        }
    }
}
impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for TimeInForce {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfTradePreventionType {
    TakerAtCross,
    Maker,
    #[serde(other)]
    Unknown,
}
impl SelfTradePreventionType {
    pub fn as_str(self) -> &'static str {
        match self {
            SelfTradePreventionType::TakerAtCross => "taker_at_cross",
            SelfTradePreventionType::Maker => "maker",
            SelfTradePreventionType::Unknown => "unknown",
        }
    }
}
impl fmt::Display for SelfTradePreventionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for SelfTradePreventionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub ticker: String,
    #[serde(default)]
    pub status: Option<OrderStatus>,
    #[serde(default)]
    pub side: Option<YesNo>,
    #[serde(default)]
    pub action: Option<BuySell>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub remaining_count: Option<i64>,
    #[serde(default)]
    pub remaining_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub filled_count: Option<i64>,
    #[serde(default)]
    pub filled_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub yes_price: Option<i64>,
    #[serde(default)]
    pub no_price: Option<i64>,
    #[serde(default)]
    #[serde(alias = "yes_price_fixed")]
    pub yes_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    #[serde(alias = "no_price_fixed")]
    pub no_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub updated_time: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default, rename = "type", alias = "order_type")]
    pub order_type: Option<OrderType>,
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    #[serde(default)]
    pub reduce_only: Option<bool>,
    #[serde(default)]
    pub post_only: Option<bool>,
    #[serde(default)]
    pub cancel_order_on_pause: Option<bool>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,
    #[serde(default)]
    pub subaccount: Option<u32>,
    #[serde(default)]
    pub fees_paid: Option<i64>,
    #[serde(default)]
    pub fees_paid_fp: Option<FixedPointDollars>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Create Order body
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateOrderRequest {
    /// required
    pub ticker: String,
    /// required: yes|no
    pub side: YesNo,
    /// required: buy|sell
    pub action: BuySell,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// Provide count or count_fp; if both provided they must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<FixedPointCount>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<OrderType>,

    /// cents 1..=99
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<u32>,

    /// fixed-point dollars strings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<FixedPointDollars>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<FixedPointDollars>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,

    /// Maximum cost in cents; when specified, order auto has FoK behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,

    /// Deprecated: use reduce_only instead; only accepts 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,

    /// If true, cancel if exchange pauses while order open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,

    /// default 0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

impl CreateOrderRequest {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if self.count.is_none() && self.count_fp.is_none() {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: must provide count or count_fp".to_string(),
            ));
        }

        if let (Some(count), Some(count_fp)) = (self.count, self.count_fp.as_deref()) {
            if let Ok(fp_val) = count_fp.parse::<f64>() {
                let count_val = count as f64;
                if (fp_val - count_val).abs() > 1e-9 {
                    return Err(KalshiError::InvalidParams(
                        "CreateOrderRequest: count and count_fp must match".to_string(),
                    ));
                }
            }
        }

        let has_yes_cents = self.yes_price.is_some();
        let has_no_cents = self.no_price.is_some();
        let has_yes_dollars = self.yes_price_dollars.is_some();
        let has_no_dollars = self.no_price_dollars.is_some();

        if has_yes_cents && has_yes_dollars {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: cannot set both yes_price and yes_price_dollars".to_string(),
            ));
        }
        if has_no_cents && has_no_dollars {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: cannot set both no_price and no_price_dollars".to_string(),
            ));
        }
        if (has_yes_cents || has_yes_dollars) && (has_no_cents || has_no_dollars) {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: cannot set both yes and no prices".to_string(),
            ));
        }

        if matches!(self.r#type, Some(OrderType::Market)) {
            if has_yes_cents || has_no_cents || has_yes_dollars || has_no_dollars {
                return Err(KalshiError::InvalidParams(
                    "CreateOrderRequest: market orders cannot include price fields".to_string(),
                ));
            }
        }

        if matches!(self.r#type, Some(OrderType::Limit))
            && !(has_yes_cents || has_no_cents || has_yes_dollars || has_no_dollars)
        {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: limit orders require a price".to_string(),
            ));
        }

        if let Some(sub) = self.subaccount {
            if sub > 32 {
                return Err(KalshiError::InvalidParams(
                    "CreateOrderRequest: subaccount must be 0..=32".to_string(),
                ));
            }
        }

        if let Some(floor) = self.sell_position_floor {
            if floor != 0 {
                return Err(KalshiError::InvalidParams(
                    "CreateOrderRequest: sell_position_floor must be 0 (deprecated)".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

/// DELETE /portfolio/orders/{order_id} supports optional query parameter subaccount
#[derive(Debug, Clone, Default, Serialize)]
pub struct CancelOrderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    pub order: Order,
    pub reduced_by: i64,
    pub reduced_by_fp: FixedPointCount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fill {
    pub fill_id: String,
    pub order_id: String,
    pub trade_id: String,
    pub ticker: String,
    #[serde(default)]
    pub market_ticker: Option<String>,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub yes_price: Option<i64>,
    #[serde(default)]
    pub no_price: Option<i64>,
    #[serde(default, alias = "yes_price_dollars")]
    pub yes_price_fixed: Option<FixedPointDollars>,
    #[serde(default, alias = "no_price_dollars")]
    pub no_price_fixed: Option<FixedPointDollars>,
    #[serde(default)]
    pub side: Option<YesNo>,
    #[serde(default)]
    pub action: Option<BuySell>,
    #[serde(default)]
    pub is_taker: Option<bool>,
    #[serde(default)]
    pub fee_cost: Option<FixedPointDollars>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub subaccount_number: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetFillsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetFillsResponse {
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settlement {
    pub settlement_id: String,
    pub ticker: String,
    #[serde(default)]
    pub market_ticker: Option<String>,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub market_result: Option<String>,
    #[serde(default)]
    pub yes_count: Option<i64>,
    #[serde(default)]
    pub yes_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub yes_total_cost: Option<FixedPointDollars>,
    #[serde(default)]
    pub no_count: Option<i64>,
    #[serde(default)]
    pub no_count_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub no_total_cost: Option<FixedPointDollars>,
    #[serde(default)]
    pub revenue: Option<FixedPointDollars>,
    #[serde(default)]
    pub settled_time: Option<String>,
    #[serde(default)]
    pub fee_cost: Option<FixedPointDollars>,
    #[serde(default)]
    pub value: Option<FixedPointDollars>,
    #[serde(default)]
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSettlementsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSettlementsResponse {
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub cursor: Option<String>,
}
