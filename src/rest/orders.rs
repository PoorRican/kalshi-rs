//! Order endpoints: orders listing, single-order lookups, create/amend/decrease/cancel,
//! batch operations, queue positions, order groups, and FCM subtrader views.
//!
//! All endpoints require authentication.

use crate::KalshiError;
use crate::rest::account::{EmptyResponse, SubaccountQueryParams};
use crate::rest::client::KalshiRestClient;
use crate::rest::pagination::{CursorPager, stream_items};
use crate::rest::portfolio::GetPositionsResponse;
use crate::types::{
    BookSide, BuySell, ErrorResponse, FixedPointCount, FixedPointDollars, OrderStatus, OrderType,
    SelfTradePreventionType, TimeInForce, YesNo, deserialize_null_as_empty_vec, serialize_csv_opt,
};
use futures::stream::Stream;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// GET /portfolio/orders query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
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

    /// Filter by exchange shard. Omit to return results from all shards. Added 2026-08-20.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

impl GetOrdersParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit
            && (limit == 0 || limit > 200)
        {
            return Err(KalshiError::InvalidParams(
                "GET /portfolio/orders: limit must be 1..=200".to_string(),
            ));
        }
        if let Some(evts) = &self.event_ticker
            && evts.len() > 10
        {
            return Err(KalshiError::InvalidParams(
                "GET /portfolio/orders: event_ticker supports up to 10 tickers".to_string(),
            ));
        }
        if let Some(sub) = self.subaccount
            && sub > 32
        {
            return Err(KalshiError::InvalidParams(
                "subaccount must be 0..=32".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub order_id: String,
    pub user_id: String,
    pub client_order_id: String,
    pub ticker: String,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `outcome_side`.
    #[serde(default)]
    pub side: Option<YesNo>,
    /// Deprecated 2026-05-07; removed ~2026-05-28. Use `book_side`.
    #[serde(default)]
    pub action: Option<BuySell>,
    /// Normalized outcome side (yes | no). Added 2026-05-07.
    #[serde(default)]
    pub outcome_side: Option<YesNo>,
    /// Normalized book side (bid | ask). Added 2026-05-07.
    #[serde(default)]
    pub book_side: Option<BookSide>,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub yes_price_dollars: FixedPointDollars,
    pub no_price_dollars: FixedPointDollars,
    pub fill_count_fp: FixedPointCount,
    pub remaining_count_fp: FixedPointCount,
    pub initial_count_fp: FixedPointCount,
    pub taker_fill_cost_dollars: FixedPointDollars,
    pub maker_fill_cost_dollars: FixedPointDollars,
    pub taker_fees_dollars: FixedPointDollars,
    pub maker_fees_dollars: FixedPointDollars,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default)]
    pub cancel_order_on_pause: Option<bool>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,
    #[serde(default, rename = "subaccount_number")]
    pub subaccount_number: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetOrdersResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
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

        if let (Some(count), Some(count_fp)) = (self.count, self.count_fp.as_deref())
            && let Ok(fp_val) = count_fp.parse::<f64>()
        {
            let count_val = count as f64;
            if (fp_val - count_val).abs() > 1e-9 {
                return Err(KalshiError::InvalidParams(
                    "CreateOrderRequest: count and count_fp must match".to_string(),
                ));
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

        if matches!(self.r#type, Some(OrderType::Market))
            && (has_yes_cents || has_no_cents || has_yes_dollars || has_no_dollars)
        {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: market orders cannot include price fields".to_string(),
            ));
        }

        if matches!(self.r#type, Some(OrderType::Limit))
            && !(has_yes_cents || has_no_cents || has_yes_dollars || has_no_dollars)
        {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: limit orders require a price".to_string(),
            ));
        }

        if let Some(sub) = self.subaccount
            && sub > 32
        {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: subaccount must be 0..=32".to_string(),
            ));
        }

        if let Some(floor) = self.sell_position_floor
            && floor != 0
        {
            return Err(KalshiError::InvalidParams(
                "CreateOrderRequest: sell_position_floor must be 0 (deprecated)".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

/// DELETE /portfolio/orders/{order_id} supports optional query parameter subaccount
#[derive(Debug, Clone, Default, Serialize)]
pub struct CancelOrderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelOrderResponse {
    pub order: Order,
    pub reduced_by: i64,
    pub reduced_by_fp: FixedPointCount,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetOrderResponse {
    pub order: Order,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AmendOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    pub ticker: String,
    pub side: YesNo,
    pub action: BuySell,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<FixedPointDollars>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<FixedPointDollars>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AmendOrderResponse {
    pub old_order: Order,
    pub order: Order,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DecreaseOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_by: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_by_fp: Option<FixedPointCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_to_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecreaseOrderResponse {
    pub order: Order,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetOrderQueuePositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

impl GetOrderQueuePositionsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetOrderQueuePositionsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub queue_positions: Vec<OrderQueuePosition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderQueuePosition {
    pub order_id: String,
    pub market_ticker: String,
    pub queue_position: i64,
    #[serde(default)]
    pub queue_position_fp: Option<FixedPointCount>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetOrderQueuePositionResponse {
    pub queue_position: i64,
    #[serde(default)]
    pub queue_position_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetOrderGroupsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub order_groups: Vec<OrderGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderGroup {
    pub id: String,
    #[serde(default)]
    pub contracts_limit: Option<i64>,
    #[serde(default)]
    pub contracts_limit_fp: Option<FixedPointCount>,
    pub is_auto_cancel_enabled: bool,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateOrderGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateOrderGroupResponse {
    pub order_group_id: String,
    /// 0 = primary account, 1–32 = subaccount. Added 2026-05-07.
    #[serde(default)]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetOrderGroupResponse {
    pub is_auto_cancel_enabled: bool,
    #[serde(default)]
    pub contracts_limit: Option<i64>,
    #[serde(default)]
    pub contracts_limit_fp: Option<FixedPointCount>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<Order>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateOrderGroupLimitRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateOrdersRequest {
    pub orders: Vec<CreateOrderRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCreateOrdersResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<BatchCreateOrdersIndividualResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCreateOrdersIndividualResponse {
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub order: Option<Order>,
    #[serde(default)]
    pub error: Option<ErrorResponse>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrdersRequestOrder {
    pub order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchCancelOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<BatchCancelOrdersRequestOrder>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCancelOrdersResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<BatchCancelOrdersIndividualResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCancelOrdersIndividualResponse {
    pub order_id: String,
    #[serde(default)]
    pub order: Option<Order>,
    pub reduced_by: i64,
    pub reduced_by_fp: FixedPointCount,
    #[serde(default)]
    pub error: Option<ErrorResponse>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

pub type GetFcmOrdersResponse = GetOrdersResponse;
pub type GetFcmPositionsResponse = GetPositionsResponse;

// ---------------------------------------------------------------------------
// V2 event-order endpoints  (/portfolio/events/orders/*)
// ---------------------------------------------------------------------------

/// Create Order (V2) body. Uses `BookSide` + single fixed-point price.
///
/// Required: `ticker`, `side`, `count`, `price`, `time_in_force`, `self_trade_prevention_type`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderV2Request {
    pub ticker: String,
    pub side: BookSide,
    pub count: FixedPointCount,
    pub price: FixedPointDollars,
    pub time_in_force: TimeInForce,
    pub self_trade_prevention_type: SelfTradePreventionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Unix seconds; combine with `time_in_force: good_till_canceled` for GTT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// 0 = primary; 1–32 = subaccount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,
    /// Exchange shard index; currently only 0 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateOrderV2Response {
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub fill_count: FixedPointCount,
    pub remaining_count: FixedPointCount,
    #[serde(default)]
    pub average_fill_price: Option<FixedPointDollars>,
    #[serde(default)]
    pub average_fee_paid: Option<FixedPointDollars>,
    pub ts_ms: i64,
}

/// Query params for Cancel Order (V2).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CancelOrderV2Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelOrderV2Response {
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub reduced_by: FixedPointCount,
    pub ts_ms: i64,
}

/// Amend Order (V2) body. Uses `BookSide` + single fixed-point price.
///
/// Required: `ticker`, `side`, `price`, `count`.
#[derive(Debug, Clone, Serialize)]
pub struct AmendOrderV2Request {
    pub ticker: String,
    pub side: BookSide,
    pub price: FixedPointDollars,
    pub count: FixedPointCount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AmendOrderV2Response {
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub remaining_count: Option<FixedPointCount>,
    #[serde(default)]
    pub fill_count: Option<FixedPointCount>,
    #[serde(default)]
    pub average_fill_price: Option<FixedPointDollars>,
    #[serde(default)]
    pub average_fee_paid: Option<FixedPointDollars>,
    pub ts_ms: i64,
}

/// Decrease Order (V2) body. Fixed-point strings only; no integer variants.
///
/// Exactly one of `reduce_by` or `reduce_to` must be provided.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DecreaseOrderV2Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_by: Option<FixedPointCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_to: Option<FixedPointCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecreaseOrderV2Response {
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub remaining_count: FixedPointCount,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateOrdersV2Request {
    pub orders: Vec<CreateOrderV2Request>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCreateOrderV2OrderResponse {
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub fill_count: Option<FixedPointCount>,
    #[serde(default)]
    pub remaining_count: Option<FixedPointCount>,
    #[serde(default)]
    pub average_fill_price: Option<FixedPointDollars>,
    #[serde(default)]
    pub average_fee_paid: Option<FixedPointDollars>,
    #[serde(default)]
    pub ts_ms: Option<i64>,
    #[serde(default)]
    pub error: Option<ErrorResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCreateOrdersV2Response {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<BatchCreateOrderV2OrderResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrderV2RequestOrder {
    pub order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchCancelOrdersV2Request {
    pub orders: Vec<BatchCancelOrderV2RequestOrder>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCancelOrderV2OrderResponse {
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub reduced_by: FixedPointCount,
    #[serde(default)]
    pub ts_ms: Option<i64>,
    #[serde(default)]
    pub error: Option<ErrorResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCancelOrdersV2Response {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<BatchCancelOrderV2OrderResponse>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetFcmOrdersParams {
    /// Required unless `client_order_ids` is supplied. Added 2026-09-03: no
    /// longer required on its own now that `client_order_ids` can be used
    /// instead (supplying both filters by both).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtrader_id: Option<String>,
    /// Client order IDs to filter by (max 100). Only orders created within
    /// the last 24 hours are searched. Required unless `subtrader_id` is
    /// supplied. Added 2026-09-03.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub client_order_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetFcmPositionsParams {
    pub subtrader_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl KalshiRestClient {
    /// List orders with optional filters. Supports cursor pagination.
    ///
    /// **Requires auth.**
    pub async fn get_orders(
        &self,
        params: GetOrdersParams,
    ) -> Result<GetOrdersResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// Place a new order.
    ///
    /// **Requires auth.**
    #[deprecated(
        since = "0.8.0",
        note = "Legacy /portfolio/orders mutation endpoints were deprecated 2026-06-18/25; \
                the exchange now rejects calls with a message to switch to the V2 endpoints. \
                Use `create_order_v2` instead."
    )]
    pub async fn create_order(
        &self,
        body: CreateOrderRequest,
    ) -> Result<CreateOrderResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders");
        body.validate()?;
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Cancel an order by ID.
    ///
    /// **Requires auth.**
    #[deprecated(
        since = "0.8.0",
        note = "Legacy /portfolio/orders mutation endpoints were deprecated 2026-06-18/25; \
                the exchange now rejects calls with a message to switch to the V2 endpoints. \
                Use `cancel_order_v2` instead."
    )]
    pub async fn cancel_order(
        &self,
        order_id: &str,
        params: CancelOrderParams,
    ) -> Result<CancelOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}"));
        self.send(
            Method::DELETE,
            &path,
            Some(&params),
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Amend an order (legacy).
    ///
    /// **Requires auth.**
    #[deprecated(
        since = "0.8.0",
        note = "Legacy /portfolio/orders mutation endpoints were deprecated 2026-06-18/25; \
                the exchange now rejects calls with a message to switch to the V2 endpoints. \
                Use `amend_order_v2` instead."
    )]
    pub async fn amend_order(
        &self,
        order_id: &str,
        body: AmendOrderRequest,
    ) -> Result<AmendOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}/amend"));
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Decrease an order's remaining count (legacy).
    ///
    /// **Requires auth.**
    #[deprecated(
        since = "0.8.0",
        note = "Legacy /portfolio/orders mutation endpoints were deprecated 2026-06-18/25; \
                the exchange now rejects calls with a message to switch to the V2 endpoints. \
                Use `decrease_order_v2` instead."
    )]
    pub async fn decrease_order(
        &self,
        order_id: &str,
        body: DecreaseOrderRequest,
    ) -> Result<DecreaseOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}/decrease"));
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<GetOrderResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Batch-create orders (legacy).
    ///
    /// **Requires auth.**
    #[deprecated(
        since = "0.8.0",
        note = "Legacy /portfolio/orders mutation endpoints were deprecated 2026-06-18/25; \
                the exchange now rejects calls with a message to switch to the V2 endpoints. \
                Use `batch_create_orders_v2` instead."
    )]
    pub async fn batch_create_orders(
        &self,
        body: BatchCreateOrdersRequest,
    ) -> Result<BatchCreateOrdersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders/batched");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Batch-cancel orders (legacy).
    ///
    /// **Requires auth.**
    #[deprecated(
        since = "0.8.0",
        note = "Legacy /portfolio/orders mutation endpoints were deprecated 2026-06-18/25; \
                the exchange now rejects calls with a message to switch to the V2 endpoints. \
                Use `batch_cancel_orders_v2` instead."
    )]
    pub async fn batch_cancel_orders(
        &self,
        body: BatchCancelOrdersRequest,
    ) -> Result<BatchCancelOrdersResponse, KalshiError> {
        let path = Self::full_path("/portfolio/orders/batched");
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Some(&body),
            true,
        )
        .await
    }

    pub async fn get_order_queue_positions(
        &self,
        params: GetOrderQueuePositionsParams,
    ) -> Result<GetOrderQueuePositionsResponse, KalshiError> {
        params.validate()?;
        let path = Self::full_path("/portfolio/orders/queue_positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_order_queue_position(
        &self,
        order_id: &str,
    ) -> Result<GetOrderQueuePositionResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/orders/{order_id}/queue_position"));
        self.send(
            Method::GET,
            &path,
            Option::<&()>::None,
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn get_order_groups(
        &self,
        params: SubaccountQueryParams,
    ) -> Result<GetOrderGroupsResponse, KalshiError> {
        let path = Self::full_path("/portfolio/order_groups");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn create_order_group(
        &self,
        body: CreateOrderGroupRequest,
    ) -> Result<CreateOrderGroupResponse, KalshiError> {
        let path = Self::full_path("/portfolio/order_groups/create");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn get_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<GetOrderGroupResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}"));
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn delete_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}"));
        self.send(
            Method::DELETE,
            &path,
            Some(&params),
            Option::<&()>::None,
            true,
        )
        .await
    }

    pub async fn update_order_group_limit(
        &self,
        order_group_id: &str,
        body: UpdateOrderGroupLimitRequest,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}/limit"));
        self.send(Method::PUT, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    pub async fn reset_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}/reset"));
        let body = EmptyResponse::default();
        self.send(Method::PUT, &path, Some(&params), Some(&body), true)
            .await
    }

    pub async fn trigger_order_group(
        &self,
        order_group_id: &str,
        params: SubaccountQueryParams,
    ) -> Result<EmptyResponse, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/order_groups/{order_group_id}/trigger"));
        let body = EmptyResponse::default();
        self.send(Method::PUT, &path, Some(&params), Some(&body), true)
            .await
    }

    // --- V2 event-order endpoints ---

    /// Place a new order via the V2 event-order endpoint.
    ///
    /// Uses `BookSide` + single fixed-point `price`. Lower rate-limit cost than legacy.
    ///
    /// **Requires auth.**
    pub async fn create_order_v2(
        &self,
        body: CreateOrderV2Request,
    ) -> Result<CreateOrderV2Response, KalshiError> {
        let path = Self::full_path("/portfolio/events/orders");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Cancel an order via the V2 event-order endpoint.
    ///
    /// **Requires auth.**
    pub async fn cancel_order_v2(
        &self,
        order_id: &str,
        params: CancelOrderV2Params,
    ) -> Result<CancelOrderV2Response, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/events/orders/{order_id}"));
        self.send(
            Method::DELETE,
            &path,
            Some(&params),
            Option::<&()>::None,
            true,
        )
        .await
    }

    /// Amend an order via the V2 event-order endpoint.
    ///
    /// **Requires auth.**
    pub async fn amend_order_v2(
        &self,
        order_id: &str,
        params: SubaccountQueryParams,
        body: AmendOrderV2Request,
    ) -> Result<AmendOrderV2Response, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/events/orders/{order_id}/amend"));
        self.send(Method::POST, &path, Some(&params), Some(&body), true)
            .await
    }

    /// Decrease an order via the V2 event-order endpoint.
    ///
    /// Provide exactly one of `reduce_by` or `reduce_to` in the body.
    ///
    /// **Requires auth.**
    pub async fn decrease_order_v2(
        &self,
        order_id: &str,
        params: SubaccountQueryParams,
        body: DecreaseOrderV2Request,
    ) -> Result<DecreaseOrderV2Response, KalshiError> {
        let path = Self::full_path(&format!("/portfolio/events/orders/{order_id}/decrease"));
        self.send(Method::POST, &path, Some(&params), Some(&body), true)
            .await
    }

    /// Submit a batch of orders via the V2 event-order endpoint.
    ///
    /// **Requires auth.**
    pub async fn batch_create_orders_v2(
        &self,
        body: BatchCreateOrdersV2Request,
    ) -> Result<BatchCreateOrdersV2Response, KalshiError> {
        let path = Self::full_path("/portfolio/events/orders/batched");
        self.send(Method::POST, &path, Option::<&()>::None, Some(&body), true)
            .await
    }

    /// Cancel a batch of orders via the V2 event-order endpoint.
    ///
    /// **Requires auth.**
    pub async fn batch_cancel_orders_v2(
        &self,
        body: BatchCancelOrdersV2Request,
    ) -> Result<BatchCancelOrdersV2Response, KalshiError> {
        let path = Self::full_path("/portfolio/events/orders/batched");
        self.send(
            Method::DELETE,
            &path,
            Option::<&()>::None,
            Some(&body),
            true,
        )
        .await
    }

    pub async fn get_fcm_orders(
        &self,
        params: GetFcmOrdersParams,
    ) -> Result<GetFcmOrdersResponse, KalshiError> {
        let path = Self::full_path("/fcm/orders");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    pub async fn get_fcm_positions(
        &self,
        params: GetFcmPositionsParams,
    ) -> Result<GetFcmPositionsResponse, KalshiError> {
        let path = Self::full_path("/fcm/positions");
        self.send(Method::GET, &path, Some(&params), Option::<&()>::None, true)
            .await
    }

    /// Create a pager for iterating over orders page by page.
    ///
    /// **Requires auth.** See [`CursorPager`].
    pub fn orders_pager(&self, params: GetOrdersParams) -> CursorPager<Order> {
        let client = self.clone();
        let base_params = params.clone();
        CursorPager::new(params.cursor.clone(), move |cursor| {
            let client = client.clone();
            let mut page_params = base_params.clone();
            page_params.cursor = cursor;
            Box::pin(async move {
                let resp = client.get_orders(page_params).await?;
                Ok((resp.orders, resp.cursor))
            })
        })
    }

    /// Stream orders one by one.
    ///
    /// **Requires auth.**
    pub fn stream_orders(
        &self,
        params: GetOrdersParams,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Order, KalshiError>> + Send {
        stream_items(self.orders_pager(params), max_items)
    }
}
