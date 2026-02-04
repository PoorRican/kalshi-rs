use crate::error::KalshiError;
use crate::types::{
    serialize_csv_opt, AnyJson, BuySell, EventStatus, FeeType, FixedPointCount, FixedPointDollars,
    MarketStatus, MveFilter, OrderStatus, OrderType, PositionCountFilter, SelfTradePreventionType,
    TimeInForce, TradeTakerSide, YesNo,
};
use serde::{Deserialize, Serialize};

/// --- Series ---

#[derive(Debug, Clone, Deserialize)]
pub struct SettlementSource {
    pub name: String,
    pub url: String,
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
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default)]
    pub contract_url: Option<String>,
    #[serde(default)]
    pub contract_terms_url: Option<String>,
    #[serde(default)]
    pub fee_type: Option<FeeType>,
    #[serde(default)]
    pub fee_multiplier: Option<i64>,
    #[serde(default)]
    pub additional_prohibitions: Vec<String>,
    #[serde(default)]
    pub product_metadata: Option<AnyJson>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub latest_event_ticker: Option<String>,
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
    pub with_milestones: Option<bool>,     // default false

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>,       // open|closed|settled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>,         // seconds since epoch
}

impl GetEventsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 200 {
                return Err(KalshiError::InvalidParams(
                    "GET /events: limit must be 1..=200".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Milestone {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventData {
    pub event_ticker: String,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub sub_title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
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
    pub custom_strike: Option<AnyJson>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<EventData>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventResponse {
    pub event: EventData,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceRange {
    #[serde(default)]
    pub min_price: Option<String>,
    #[serde(default)]
    pub max_price: Option<String>,
    #[serde(default)]
    pub increment: Option<String>,
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
    pub yes_ask: Option<i64>,
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
    pub liquidity: Option<i64>,
    #[serde(default)]
    pub liquidity_fp: Option<String>,
    #[serde(default)]
    pub custom_strike: Option<AnyJson>,
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<MveSelectedLeg>>,
    #[serde(default)]
    pub price_ranges: Option<Vec<AnyJson>>,
}

/// GET /markets query params and constraints
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Event tickers comma-separated (max 10)
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
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
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub tickers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

impl GetMarketsParams {
    pub fn validate(&self) -> Result<(), KalshiError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 1000 {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: limit must be 1..=1000".to_string(),
                ));
            }
        }
        if let Some(evts) = &self.event_ticker {
            if evts.len() > 10 {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: event_ticker supports up to 10 tickers".to_string(),
                ));
            }
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
                    "GET /markets: with min_updated_ts, only mve_filter=exclude is allowed".to_string(),
                ));
            }
        }

        if created {
            if matches!(self.status, Some(MarketStatus::Closed | MarketStatus::Settled | MarketStatus::Paused)) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: created_ts filters are only compatible with status unopened/open or no status".to_string(),
                ));
            }
        }
        if close {
            if matches!(self.status, Some(MarketStatus::Unopened | MarketStatus::Open | MarketStatus::Settled | MarketStatus::Paused)) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: close_ts filters are only compatible with status closed or no status".to_string(),
                ));
            }
        }
        if settled {
            if matches!(self.status, Some(MarketStatus::Unopened | MarketStatus::Open | MarketStatus::Closed | MarketStatus::Paused)) {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: settled_ts filters are only compatible with status settled or no status".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketsResponse {
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

/// --- Trades ---

#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub ticker: String,
    #[serde(default)]
    pub price: Option<i64>,
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
    #[serde(default)]
    pub standard_hours: Vec<StandardHours>,
    #[serde(default)]
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
    pub balance: i64,
    pub updated_ts: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSubaccountBalancesResponse {
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
    pub subaccount_transfers: Vec<SubaccountTransfer>,
    #[serde(default)]
    pub cursor: Option<String>,
}
