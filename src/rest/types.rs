use crate::error::KalshiError;
use crate::types::{
    BuySell, ErrorResponse, EventStatus, FeeType, FixedPointCount, FixedPointDollars, MarketStatus,
    MveFilter, OrderStatus, OrderType, PositionCountFilter, SelfTradePreventionType, TimeInForce,
    TradeTakerSide, YesNo, deserialize_null_as_empty_vec, deserialize_string_or_number,
    serialize_csv_opt,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// --- Series ---

#[derive(Debug, Clone, Deserialize)]
pub struct SettlementSource {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketMetadata {
    pub market_ticker: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub color_code: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventMetadata {
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub featured_image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_details: Vec<MarketMetadata>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default)]
    pub competition: Option<String>,
    #[serde(default)]
    pub competition_scope: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Series {
    pub ticker: String,
    #[serde(default)]
    pub frequency: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default)]
    pub contract_url: Option<String>,
    #[serde(default)]
    pub contract_terms_url: Option<String>,
    #[serde(default)]
    pub fee_type: Option<FeeType>,
    #[serde(default)]
    pub fee_multiplier: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub additional_prohibitions: Vec<String>,
    #[serde(default)]
    pub product_metadata: Option<Map<String, Value>>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub latest_event_ticker: Option<String>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub inactive: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSeriesListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Docs: "tags" is a string (not explicitly CSV-typed), so keep as raw string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_product_metadata: Option<bool>,
    /// If true, includes total volume traded across all events in each series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_volume: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesListResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub series: Vec<Series>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesResponse {
    pub series: Series,
}

/// --- Events ---
/// GET /events query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 200, max 200
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_milestones: Option<bool>, // default false

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>, // open|closed|settled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>, // seconds since epoch
}

impl GetEventsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit
            && (limit == 0 || limit > 200)
        {
            return Err(KalshiError::InvalidParams(
                "GET /events: limit must be 1..=200".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Milestone {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default, rename = "type")]
    pub milestone_type: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub related_event_tickers: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub notification_message: Option<String>,
    #[serde(default)]
    pub details: Option<Map<String, Value>>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub primary_event_tickers: Vec<String>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub source_ids: Option<Map<String, Value>>,
    // Legacy/alternative shape fields still observed in some payloads.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventData {
    pub event_ticker: String,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub collateral_return_type: Option<String>,
    #[serde(default)]
    pub mutually_exclusive: Option<bool>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub sub_title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub available_on_brokers: Option<bool>,
    #[serde(default)]
    pub strike_date: Option<String>,
    #[serde(default)]
    pub strike_period: Option<String>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub can_trade: Option<bool>,
    #[serde(default)]
    pub can_settle: Option<bool>,
    #[serde(default)]
    pub start_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default)]
    pub settled_ts: Option<i64>,
    #[serde(default)]
    pub series_id: Option<i64>,
    #[serde(default)]
    pub mutual_exclusive_group_id: Option<String>,
    #[serde(default)]
    pub mutual_exclusive_group_ids: Option<Vec<String>>,
    #[serde(default)]
    pub event_delta: Option<i64>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub markets: Option<Vec<Market>>,
    #[serde(default)]
    pub milestones: Option<Vec<Milestone>>,
    #[serde(default)]
    pub custom_strike: Option<Map<String, Value>>,
    #[serde(default)]
    pub product_metadata: Option<EventMetadata>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<EventData>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventResponse {
    pub event: EventData,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub markets: Vec<Market>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// --- Markets ---

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketState {
    Initialized,
    Inactive,
    Active,
    Closed,
    Determined,
    Disputed,
    Amended,
    Finalized,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MveSelectedLeg {
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub market_ticker: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub yes_settlement_value_dollars: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceRange {
    #[serde(alias = "min_price")]
    pub start: String,
    #[serde(alias = "max_price")]
    pub end: String,
    #[serde(alias = "increment")]
    pub step: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Market {
    pub ticker: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub market_id: Option<String>,
    #[serde(default)]
    pub status: Option<MarketState>,
    #[serde(default)]
    pub market_type: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub event_title: Option<String>,
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub no_sub_title: Option<String>,
    #[serde(default)]
    pub rules_primary: Option<String>,
    #[serde(default)]
    pub rules_secondary: Option<String>,
    #[serde(default)]
    pub resolution_source: Option<String>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub can_trade: Option<bool>,
    #[serde(default)]
    pub can_settle: Option<bool>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub series_id: Option<i64>,
    #[serde(default)]
    pub event_id: Option<i64>,
    #[serde(default)]
    pub response_price_units: Option<String>,
    #[serde(default)]
    pub settlement_timer_seconds: Option<i64>,
    #[serde(default)]
    pub price_level_structure: Option<String>,
    #[serde(default)]
    pub open_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default)]
    pub settled_ts: Option<i64>,
    #[serde(default)]
    pub expiration_ts: Option<i64>,
    #[serde(default)]
    pub open_time: Option<String>,
    #[serde(default)]
    pub close_time: Option<String>,
    #[serde(default)]
    pub expected_expiration_time: Option<String>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub latest_expiration_time: Option<String>,
    #[serde(default)]
    pub created_ts: Option<i64>,
    #[serde(default)]
    pub updated_ts: Option<i64>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub updated_time: Option<String>,
    #[serde(default)]
    pub floor_price: Option<i64>,
    #[serde(default)]
    pub cap_price: Option<i64>,
    #[serde(default)]
    pub yes_bid: Option<i64>,
    #[serde(default)]
    pub yes_bid_size_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub yes_ask: Option<i64>,
    #[serde(default)]
    pub yes_ask_size_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub no_bid: Option<i64>,
    #[serde(default)]
    pub no_ask: Option<i64>,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default)]
    pub last_price: Option<i64>,
    #[serde(default)]
    pub yes_bid_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub yes_ask_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub no_bid_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub no_ask_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub last_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub volume_24h: Option<i64>,
    #[serde(default)]
    pub volume_24h_fp: Option<String>,
    #[serde(default)]
    pub open_interest: Option<i64>,
    #[serde(default)]
    pub open_interest_fp: Option<String>,
    #[serde(default)]
    pub fractional_trading_enabled: Option<bool>,
    #[serde(default)]
    pub notional_value: Option<i64>,
    #[serde(default)]
    pub notional_value_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub previous_yes_bid: Option<i64>,
    #[serde(default)]
    pub previous_yes_bid_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub previous_yes_ask: Option<i64>,
    #[serde(default)]
    pub previous_yes_ask_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub previous_price: Option<i64>,
    #[serde(default)]
    pub previous_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub liquidity: Option<i64>,
    #[serde(default)]
    pub liquidity_fp: Option<String>,
    #[serde(default)]
    pub liquidity_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub expiration_value: Option<String>,
    #[serde(default)]
    pub tick_size: Option<i64>,
    #[serde(default)]
    pub settlement_value: Option<i64>,
    #[serde(default)]
    pub settlement_value_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub settlement_ts: Option<String>,
    #[serde(default)]
    pub fee_waiver_expiration_time: Option<String>,
    #[serde(default)]
    pub early_close_condition: Option<String>,
    #[serde(default)]
    pub strike_type: Option<String>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
    #[serde(default)]
    pub functional_strike: Option<String>,
    #[serde(default)]
    pub custom_strike: Option<Map<String, Value>>,
    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<MveSelectedLeg>>,
    #[serde(default)]
    pub primary_participant_key: Option<String>,
    #[serde(default)]
    pub is_provisional: Option<bool>,
    #[serde(default)]
    pub price_ranges: Option<Vec<PriceRange>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// GET /markets query params and constraints
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Event tickers comma-separated (max 10)
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub event_ticker: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_created_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_created_ts: Option<i64>,

    /// min_updated_ts is incompatible with any other filters besides mve_filter=exclude.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_updated_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_close_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_settled_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_settled_ts: Option<i64>,

    /// Only one status filter may be supplied at a time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,

    /// Market tickers comma-separated.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub tickers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

impl GetMarketsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit
            && (limit == 0 || limit > 1000)
        {
            return Err(KalshiError::InvalidParams(
                "GET /markets: limit must be 1..=1000".to_string(),
            ));
        }
        if let Some(evts) = &self.event_ticker
            && evts.len() > 10
        {
            return Err(KalshiError::InvalidParams(
                "GET /markets: event_ticker supports up to 10 tickers".to_string(),
            ));
        }

        let created = self.min_created_ts.is_some() || self.max_created_ts.is_some();
        let close = self.min_close_ts.is_some() || self.max_close_ts.is_some();
        let settled = self.min_settled_ts.is_some() || self.max_settled_ts.is_some();
        let updated = self.min_updated_ts.is_some();

        let groups = [created, close, settled, updated]
            .iter()
            .filter(|x| **x)
            .count();
        if groups > 1 {
            return Err(KalshiError::InvalidParams(
                "GET /markets: timestamp filters are mutually exclusive (created vs close vs settled vs updated)"
                    .to_string(),
            ));
        }

        if updated {
            if self.status.is_some()
                || self.series_ticker.is_some()
                || self.event_ticker.is_some()
                || self.tickers.is_some()
                || created
                || close
                || settled
            {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: min_updated_ts cannot be combined with other filters (except mve_filter=exclude)"
                        .to_string(),
                ));
            }
            if matches!(self.mve_filter, Some(MveFilter::Only)) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: with min_updated_ts, only mve_filter=exclude is allowed"
                        .to_string(),
                ));
            }
        }

        if created
            && matches!(
                self.status,
                Some(MarketStatus::Closed | MarketStatus::Settled | MarketStatus::Paused)
            )
        {
            return Err(KalshiError::InvalidParams(
                    "GET /markets: created_ts filters are only compatible with status unopened/open or no status".to_string(),
                ));
        }
        if close
            && matches!(
                self.status,
                Some(
                    MarketStatus::Unopened
                        | MarketStatus::Open
                        | MarketStatus::Settled
                        | MarketStatus::Paused
                )
            )
        {
            return Err(KalshiError::InvalidParams(
                    "GET /markets: close_ts filters are only compatible with status closed or no status".to_string(),
                ));
        }
        if settled
            && matches!(
                self.status,
                Some(
                    MarketStatus::Unopened
                        | MarketStatus::Open
                        | MarketStatus::Closed
                        | MarketStatus::Paused
                )
            )
        {
            return Err(KalshiError::InvalidParams(
                    "GET /markets: settled_ts filters are only compatible with status settled or no status".to_string(),
                ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub markets: Vec<Market>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketResponse {
    pub market: Market,
}

/// --- Orderbook ---

#[derive(Debug, Clone, Deserialize)]
pub struct Orderbook {
    /// Price levels: (price_cents, quantity)
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub yes: Vec<(i64, i64)>,
    /// Price levels: (price_cents, quantity)
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub no: Vec<(i64, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub yes_dollars: Vec<(FixedPointDollars, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub no_dollars: Vec<(FixedPointDollars, i64)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookFp {
    /// Price levels: (price_dollars, quantity_fp)
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub yes_dollars: Vec<(FixedPointDollars, String)>,
    /// Price levels: (price_dollars, quantity_fp)
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
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

/// --- Trades ---

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub ticker: String,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub count_fp: Option<String>,
    #[serde(default)]
    pub yes_price: Option<i64>,
    #[serde(default)]
    pub no_price: Option<i64>,
    #[serde(default)]
    pub yes_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub no_price_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub taker_side: Option<TradeTakerSide>,
    #[serde(default)]
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetTradesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTradesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub trades: Vec<Trade>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// --- Exchange ---

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeStatusResponse {
    pub exchange_active: bool,
    pub trading_active: bool,
    #[serde(default)]
    pub exchange_estimated_resume_time: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementType {
    Info,
    Warning,
    Error,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementStatus {
    Active,
    Inactive,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Announcement {
    #[serde(rename = "type")]
    pub r#type: AnnouncementType,
    pub message: String,
    pub delivery_time: String,
    pub status: AnnouncementStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeAnnouncementsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub announcements: Vec<Announcement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailySchedule {
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StandardHours {
    pub start_time: String,
    pub end_time: String,
    #[serde(default)]
    pub monday: Vec<DailySchedule>,
    #[serde(default)]
    pub tuesday: Vec<DailySchedule>,
    #[serde(default)]
    pub wednesday: Vec<DailySchedule>,
    #[serde(default)]
    pub thursday: Vec<DailySchedule>,
    #[serde(default)]
    pub friday: Vec<DailySchedule>,
    #[serde(default)]
    pub saturday: Vec<DailySchedule>,
    #[serde(default)]
    pub sunday: Vec<DailySchedule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaintenanceWindow {
    pub start_datetime: String,
    pub end_datetime: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeSchedule {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub standard_hours: Vec<StandardHours>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetExchangeScheduleResponse {
    pub schedule: ExchangeSchedule,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserDataTimestampResponse {
    pub as_of_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeriesFeeChange {
    pub id: i64,
    pub series_ticker: String,
    pub fee_type: FeeType,
    pub fee_multiplier: i64,
    pub scheduled_ts: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSeriesFeeChangesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_historical: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesFeeChangesResponse {
    #[serde(rename = "series_fee_change_arr")]
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub series_fee_change_arr: Vec<SeriesFeeChange>,
}

/// --- Portfolio / Orders ---

#[derive(Debug, Clone, Deserialize)]
pub struct GetBalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
    pub updated_ts: i64,
}

/// GET /portfolio/positions query params
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetPositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000

    /// CSV of non-zero filters (position,total_traded)
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub count_filter: Option<Vec<PositionCountFilter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_csv_opt"
    )]
    pub event_ticker: Option<Vec<String>>,

    /// 0..=32
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

impl GetPositionsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit
            && (limit == 0 || limit > 1000)
        {
            return Err(KalshiError::InvalidParams(
                "GET /portfolio/positions: limit must be 1..=1000".to_string(),
            ));
        }
        if let Some(evts) = &self.event_ticker
            && evts.len() > 10
        {
            return Err(KalshiError::InvalidParams(
                "GET /portfolio/positions: event_ticker supports up to 10 tickers".to_string(),
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
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_positions: Vec<MarketPosition>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub event_positions: Vec<EventPosition>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PositionsPage {
    pub market_positions: Vec<MarketPosition>,
    pub event_positions: Vec<EventPosition>,
}

impl From<GetPositionsResponse> for PositionsPage {
    fn from(resp: GetPositionsResponse) -> Self {
        Self {
            market_positions: resp.market_positions,
            event_positions: resp.event_positions,
        }
    }
}

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
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
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
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// --- Account ---

#[derive(Debug, Clone, Deserialize)]
pub struct GetAccountApiLimitsResponse {
    pub usage_tier: String,
    pub read_limit: i64,
    pub write_limit: i64,
}

/// --- Subaccounts ---

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSubaccountResponse {
    pub subaccount_number: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubaccountBalance {
    pub subaccount_number: u32,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub balance: FixedPointDollars,
    pub updated_ts: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSubaccountBalancesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub subaccount_balances: Vec<SubaccountBalance>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplySubaccountTransferRequest {
    pub client_transfer_id: String,
    pub from_subaccount: u32,
    pub to_subaccount: u32,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApplySubaccountTransferResponse {}

#[derive(Debug, Clone, Deserialize)]
pub struct SubaccountTransfer {
    pub transfer_id: String,
    pub from_subaccount: u32,
    pub to_subaccount: u32,
    pub amount_cents: i64,
    pub created_ts: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSubaccountTransfersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSubaccountTransfersResponse {
    #[serde(
        default,
        deserialize_with = "deserialize_null_as_empty_vec",
        alias = "subaccount_transfer_arr",
        alias = "transfers"
    )]
    pub subaccount_transfers: Vec<SubaccountTransfer>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// --- Additional OpenAPI v3.7.0 Models ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenericObject {
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmptyResponse {}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiKey {
    pub api_key_id: String,
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub scopes: Vec<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// GET /api_keys
#[derive(Debug, Clone, Deserialize)]
pub struct GetApiKeysResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub api_keys: Vec<ApiKey>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub public_key: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key_id: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateApiKeyRequest {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateApiKeyResponse {
    pub api_key_id: String,
    pub private_key: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetCommunicationsIdResponse {
    pub communications_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Quote {
    pub id: String,
    pub rfq_id: String,
    pub creator_id: String,
    #[serde(default)]
    pub rfq_creator_id: Option<String>,
    pub market_ticker: String,
    pub contracts: i64,
    pub contracts_fp: FixedPointCount,
    pub yes_bid: i64,
    pub no_bid: i64,
    pub yes_bid_dollars: FixedPointDollars,
    pub no_bid_dollars: FixedPointDollars,
    pub created_ts: String,
    pub updated_ts: String,
    pub status: String,
    #[serde(default)]
    pub accepted_side: Option<YesNo>,
    #[serde(default)]
    pub accepted_ts: Option<String>,
    #[serde(default)]
    pub confirmed_ts: Option<String>,
    #[serde(default)]
    pub executed_ts: Option<String>,
    #[serde(default)]
    pub cancelled_ts: Option<String>,
    #[serde(default)]
    pub rest_remainder: Option<bool>,
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    #[serde(default)]
    pub creator_user_id: Option<String>,
    #[serde(default)]
    pub rfq_creator_user_id: Option<String>,
    #[serde(default)]
    pub rfq_target_cost_centi_cents: Option<i64>,
    #[serde(default)]
    pub rfq_target_cost_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub rfq_creator_order_id: Option<String>,
    #[serde(default)]
    pub creator_order_id: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RFQ {
    pub id: String,
    pub creator_id: String,
    pub market_ticker: String,
    pub contracts: i64,
    pub contracts_fp: FixedPointCount,
    #[serde(default)]
    pub target_cost_centi_cents: Option<i64>,
    #[serde(default)]
    pub target_cost_dollars: Option<FixedPointDollars>,
    pub status: String,
    pub created_ts: String,
    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub mve_selected_legs: Vec<MveSelectedLeg>,
    #[serde(default)]
    pub rest_remainder: Option<bool>,
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    #[serde(default)]
    pub creator_user_id: Option<String>,
    #[serde(default)]
    pub cancelled_ts: Option<String>,
    #[serde(default)]
    pub updated_ts: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetQuotesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_creator_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_creator_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_creator_subtrader_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetQuotesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub quotes: Vec<Quote>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetQuoteResponse {
    pub quote: Quote,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateQuoteRequest {
    pub rfq_id: String,
    pub yes_bid: String,
    pub no_bid: String,
    pub rest_remainder: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateQuoteResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptQuoteRequest {
    pub accepted_side: YesNo,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetRFQsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetRFQsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub rfqs: Vec<RFQ>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetRFQResponse {
    pub rfq: RFQ,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateRFQRequest {
    pub market_ticker: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_fp: Option<FixedPointCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_centi_cents: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_dollars: Option<FixedPointDollars>,
    pub rest_remainder: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_existing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtrader_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRFQResponse {
    pub id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMultivariateEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMultivariateEventsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub events: Vec<EventData>,
    #[serde(default)]
    pub cursor: Option<String>,
}

pub type GetEventMetadataResponse = EventMetadata;

pub type GetFcmOrdersResponse = GetOrdersResponse;
pub type GetFcmPositionsResponse = GetPositionsResponse;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetFcmOrdersParams {
    pub subtrader_id: String,
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetIncentiveProgramsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub incentive_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetIncentiveProgramsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub incentive_programs: Vec<IncentiveProgram>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncentiveProgram {
    pub id: String,
    pub market_id: String,
    pub market_ticker: String,
    pub incentive_type: String,
    pub start_date: String,
    pub end_date: String,
    pub period_reward: i64,
    pub paid_out: bool,
    #[serde(default)]
    pub discount_factor_bps: Option<i32>,
    #[serde(default)]
    pub target_size: Option<i32>,
    #[serde(default)]
    pub target_size_fp: Option<FixedPointCount>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetLiveDatasParams {
    pub milestone_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetLiveDatasResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub live_datas: Vec<LiveData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetLiveDataResponse {
    pub live_data: LiveData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiveData {
    #[serde(rename = "type")]
    pub live_data_type: String,
    #[serde(default)]
    pub details: Map<String, Value>,
    pub milestone_id: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchGetMarketCandlesticksParams {
    pub market_tickers: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_latest_before_start: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BidAskDistribution {
    #[serde(default)]
    pub open: Option<i64>,
    #[serde(default)]
    pub open_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub low: Option<i64>,
    #[serde(default)]
    pub low_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub high: Option<i64>,
    #[serde(default)]
    pub high_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub close: Option<i64>,
    #[serde(default)]
    pub close_dollars: Option<FixedPointDollars>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceDistribution {
    #[serde(default)]
    pub open: Option<i64>,
    #[serde(default)]
    pub open_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub low: Option<i64>,
    #[serde(default)]
    pub low_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub high: Option<i64>,
    #[serde(default)]
    pub high_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub close: Option<i64>,
    #[serde(default)]
    pub close_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub mean: Option<i64>,
    #[serde(default)]
    pub mean_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub previous: Option<i64>,
    #[serde(default)]
    pub previous_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub min_dollars: Option<FixedPointDollars>,
    #[serde(default)]
    pub max: Option<i64>,
    #[serde(default)]
    pub max_dollars: Option<FixedPointDollars>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketCandlestick {
    pub end_period_ts: i64,
    pub yes_bid: BidAskDistribution,
    pub yes_ask: BidAskDistribution,
    pub price: PriceDistribution,
    pub volume: i64,
    pub volume_fp: FixedPointCount,
    pub open_interest: i64,
    pub open_interest_fp: FixedPointCount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketCandlesticksResponse {
    pub market_ticker: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub candlesticks: Vec<MarketCandlestick>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchGetMarketCandlesticksResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub markets: Vec<MarketCandlesticksResponse>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMilestonesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub milestone_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMilestonesResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMilestoneResponse {
    pub milestone: Milestone,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMultivariateEventCollectionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMultivariateEventCollectionsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub multivariate_contracts: Vec<MultivariateEventCollection>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMultivariateEventCollectionResponse {
    pub multivariate_contract: MultivariateEventCollection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssociatedEvent {
    pub ticker: String,
    pub is_yes_only: bool,
    #[serde(default)]
    pub size_max: Option<i32>,
    #[serde(default)]
    pub size_min: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub active_quoters: Vec<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultivariateEventCollection {
    pub collection_ticker: String,
    pub series_ticker: String,
    pub title: String,
    pub description: String,
    pub open_date: String,
    pub close_date: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub associated_events: Vec<AssociatedEvent>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub associated_event_tickers: Vec<String>,
    pub is_ordered: bool,
    pub is_single_market_per_event: bool,
    pub is_all_yes: bool,
    pub size_min: i32,
    pub size_max: i32,
    pub functional_description: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerPair {
    pub market_ticker: String,
    pub event_ticker: String,
    pub side: YesNo,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateMarketInMultivariateEventCollectionRequest {
    pub selected_markets: Vec<TickerPair>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_market_payload: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMarketInMultivariateEventCollectionResponse {
    pub event_ticker: String,
    pub market_ticker: String,
    #[serde(default)]
    pub market: Option<Market>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMultivariateEventCollectionLookupHistoryParams {
    pub lookback_seconds: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMultivariateEventCollectionLookupHistoryResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub lookup_points: Vec<LookupPoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LookupPoint {
    pub event_ticker: String,
    pub market_ticker: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub selected_markets: Vec<TickerPair>,
    pub last_queried_ts: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LookupTickersForMarketInMultivariateEventCollectionRequest {
    pub selected_markets: Vec<TickerPair>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LookupTickersForMarketInMultivariateEventCollectionResponse {
    pub event_ticker: String,
    pub market_ticker: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SubaccountQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderGroupsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub order_groups: Vec<OrderGroup>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderGroupResponse {
    pub order_group_id: String,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct BatchCreateOrdersResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<BatchCreateOrdersIndividualResponse>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct BatchCancelOrdersResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub orders: Vec<BatchCancelOrdersIndividualResponse>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderQueuePositionsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub queue_positions: Vec<OrderQueuePosition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderQueuePosition {
    pub order_id: String,
    pub market_ticker: String,
    pub queue_position: i64,
    #[serde(default)]
    pub queue_position_fp: Option<FixedPointCount>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetOrderQueuePositionResponse {
    pub queue_position: i64,
    #[serde(default)]
    pub queue_position_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPortfolioRestingOrderTotalValueResponse {
    pub total_resting_order_value: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTagsForSeriesCategoriesResponse {
    #[serde(default)]
    pub tags_by_categories: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetFiltersBySportsResponse {
    #[serde(default)]
    pub filters_by_sports: Map<String, Value>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub sport_ordering: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetMarketCandlesticksParams {
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_latest_before_start: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetMarketCandlesticksHistoricalParams {
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tickers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalFillsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetHistoricalOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetEventCandlesticksParams {
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetEventForecastPercentileHistoryParams {
    pub percentiles: Vec<u32>,
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketCandlesticksResponse {
    pub ticker: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub candlesticks: Vec<MarketCandlestick>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BidAskDistributionHistorical {
    pub open: FixedPointDollars,
    pub low: FixedPointDollars,
    pub high: FixedPointDollars,
    pub close: FixedPointDollars,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceDistributionHistorical {
    #[serde(default)]
    pub open: Option<FixedPointDollars>,
    #[serde(default)]
    pub low: Option<FixedPointDollars>,
    #[serde(default)]
    pub high: Option<FixedPointDollars>,
    #[serde(default)]
    pub close: Option<FixedPointDollars>,
    #[serde(default)]
    pub mean: Option<FixedPointDollars>,
    #[serde(default)]
    pub previous: Option<FixedPointDollars>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketCandlestickHistorical {
    pub end_period_ts: i64,
    pub yes_bid: BidAskDistributionHistorical,
    pub yes_ask: BidAskDistributionHistorical,
    pub price: PriceDistributionHistorical,
    pub volume: FixedPointCount,
    pub open_interest: FixedPointCount,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketCandlesticksHistoricalResponse {
    pub ticker: String,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub candlesticks: Vec<MarketCandlestickHistorical>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHistoricalCutoffResponse {
    pub market_settled_ts: String,
    pub trades_created_ts: String,
    pub orders_updated_ts: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventCandlesticksResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_tickers: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub market_candlesticks: Vec<Vec<MarketCandlestick>>,
    pub adjusted_end_ts: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventForecastPercentilesHistoryResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub forecast_history: Vec<ForecastPercentilesPoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastPercentilesPoint {
    pub event_ticker: String,
    pub end_period_ts: i64,
    pub period_interval: i32,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub percentile_points: Vec<PercentilePoint>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PercentilePoint {
    pub percentile: i32,
    pub raw_numerical_forecast: f64,
    pub numerical_forecast: f64,
    pub formatted_forecast: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetStructuredTargetsParams {
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub target_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStructuredTargetsResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub structured_targets: Vec<StructuredTarget>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStructuredTargetResponse {
    pub structured_target: StructuredTarget,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StructuredTarget {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "type")]
    pub target_type: Option<String>,
    #[serde(default)]
    pub details: Option<Map<String, Value>>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub source_ids: Option<Map<String, Value>>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubaccountNettingRequest {
    pub subaccount_number: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountNettingConfig {
    pub subaccount_number: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSubaccountNettingResponse {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub netting_configs: Vec<SubaccountNettingConfig>,
}
