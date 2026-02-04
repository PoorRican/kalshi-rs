use crate::error::KalshiError;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

/// Serialize Option<Vec<T>> as a single comma-separated query param
fn serialize_csv_opt<T, S>(value: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(items) => {
            let s = items.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
            serializer.serialize_str(&s)
        }
    }
}

/// --- Series ---

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetSeriesListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Docs: "tags" is a string (not explicitly CSV-typed), so keep as raw string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_product_metadata: Option<bool>,
    /// If true, includes total volume traded across all events in each series. :contentReference[oaicite:36]{index=36}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_volume: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetSeriesListResponse {
    pub series: Vec<serde_json::Value>,
}

/// --- Events ---

#[derive(Debug, Clone, Copy)]
pub enum EventStatus {
    Open,
    Closed,
    Settled,
}

impl EventStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            EventStatus::Open => "open",
            EventStatus::Closed => "closed",
            EventStatus::Settled => "settled",
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for EventStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// GET /events query params :contentReference[oaicite:37]{index=37}
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

/// Response shape in adapter: keep payload flexible.
#[derive(Debug, Clone, Deserialize)]
pub struct GetEventsResponse {
    pub events: Vec<serde_json::Value>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>, // default false :contentReference[oaicite:38]{index=38}
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEventResponse {
    pub event: serde_json::Value,
}

/// --- Markets ---

#[derive(Debug, Clone, Copy)]
pub enum MarketStatus {
    Unopened,
    Open,
    Paused,
    Closed,
    Settled,
}

impl MarketStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MarketStatus::Unopened => "unopened",
            MarketStatus::Open => "open",
            MarketStatus::Paused => "paused",
            MarketStatus::Closed => "closed",
            MarketStatus::Settled => "settled",
        }
    }
}

impl fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for MarketStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MveFilter {
    Only,
    Exclude,
}

impl MveFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            MveFilter::Only => "only",
            MveFilter::Exclude => "exclude",
        }
    }
}

impl fmt::Display for MveFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for MveFilter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// GET /markets query params and constraints :contentReference[oaicite:39]{index=39}
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Event tickers comma-separated (max 10) :contentReference[oaicite:40]{index=40}
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub event_ticker: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_created_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_created_ts: Option<i64>,

    /// Note in docs: min_updated_ts is incompatible with any other filters besides mve_filter=exclude :contentReference[oaicite:41]{index=41}
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

    /// Only one status filter may be supplied at a time :contentReference[oaicite:42]{index=42}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketStatus>,

    /// Market tickers comma-separated :contentReference[oaicite:43]{index=43}
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

        // Timestamp filter compatibility rules :contentReference[oaicite:44]{index=44}
        let created = self.min_created_ts.is_some() || self.max_created_ts.is_some();
        let close = self.min_close_ts.is_some() || self.max_close_ts.is_some();
        let settled = self.min_settled_ts.is_some() || self.max_settled_ts.is_some();
        let updated = self.min_updated_ts.is_some();

        let groups = [created, close, settled, updated].iter().filter(|x| **x).count();
        if groups > 1 {
            return Err(KalshiError::InvalidParams(
                "GET /markets: timestamp filters are mutually exclusive (created vs close vs settled vs updated)".to_string(),
            ));
        }

        if updated {
            // "Incompatible with any other filters besides mve_filter=exclude" :contentReference[oaicite:45]{index=45}
            if self.status.is_some()
                || self.series_ticker.is_some()
                || self.event_ticker.is_some()
                || self.tickers.is_some()
                || created
                || close
                || settled
            {
                return Err(KalshiError::InvalidParams(
                    "GET /markets: min_updated_ts cannot be combined with other filters (except mve_filter=exclude)".to_string(),
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
    pub markets: Vec<serde_json::Value>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketResponse {
    pub market: serde_json::Value,
}

/// --- Portfolio / Orders ---

#[derive(Debug, Clone, Deserialize)]
pub struct GetBalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
    pub updated_ts: i64,
}

/// count_filter accepted values: position, total_traded :contentReference[oaicite:46]{index=46}
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

/// GET /portfolio/positions query params :contentReference[oaicite:47]{index=47}
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetPositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // default 100, max 1000

    /// CSV of non-zero filters (position,total_traded) :contentReference[oaicite:48]{index=48}
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub count_filter: Option<Vec<PositionCountFilter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10 :contentReference[oaicite:49]{index=49}
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_csv_opt")]
    pub event_ticker: Option<Vec<String>>,

    /// 0..=32 :contentReference[oaicite:50]{index=50}
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
pub struct GetPositionsResponse {
    pub market_positions: Vec<serde_json::Value>,
    pub event_positions: Vec<serde_json::Value>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Resting,
    Canceled,
    Executed,
}

impl OrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderStatus::Resting => "resting",
            OrderStatus::Canceled => "canceled",
            OrderStatus::Executed => "executed",
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

/// GET /portfolio/orders query params :contentReference[oaicite:51]{index=51}
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// CSV max 10 :contentReference[oaicite:52]{index=52}
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
pub struct GetOrdersResponse {
    pub orders: Vec<serde_json::Value>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Create Order body :contentReference[oaicite:53]{index=53}
#[derive(Debug, Clone, Copy)]
pub enum YesNo {
    Yes,
    No,
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

#[derive(Debug, Clone, Copy)]
pub enum BuySell {
    Buy,
    Sell,
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

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Limit,
    Market,
}
impl OrderType {
    pub fn as_str(self) -> &'static str {
        match self {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
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

#[derive(Debug, Clone, Copy)]
pub enum TimeInForce {
    FillOrKill,
    GoodTillCanceled,
    ImmediateOrCancel,
}
impl TimeInForce {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeInForce::FillOrKill => "fill_or_kill",
            TimeInForce::GoodTillCanceled => "good_till_canceled",
            TimeInForce::ImmediateOrCancel => "immediate_or_cancel",
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

#[derive(Debug, Clone, Copy)]
pub enum SelfTradePreventionType {
    TakerAtCross,
    Maker,
}
impl SelfTradePreventionType {
    pub fn as_str(self) -> &'static str {
        match self {
            SelfTradePreventionType::TakerAtCross => "taker_at_cross",
            SelfTradePreventionType::Maker => "maker",
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateOrderRequest {
    /// required :contentReference[oaicite:54]{index=54}
    pub ticker: String,
    /// required: yes|no :contentReference[oaicite:55]{index=55}
    pub side: YesNo,
    /// required: buy|sell :contentReference[oaicite:56]{index=56}
    pub action: BuySell,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// Provide count or count_fp; if both provided they must match :contentReference[oaicite:57]{index=57}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<OrderType>, // docs list values limit|market :contentReference[oaicite:58]{index=58}

    /// cents 1..=99 :contentReference[oaicite:59]{index=59}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<u32>,

    /// fixed-point dollars strings :contentReference[oaicite:60]{index=60}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,

    /// Maximum cost in cents; when specified, order auto has FoK behavior :contentReference[oaicite:61]{index=61}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,

    /// Deprecated: use reduce_only instead; only accepts 0 :contentReference[oaicite:62]{index=62}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,

    /// If true, cancel if exchange pauses while order open :contentReference[oaicite:63]{index=63}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,

    /// default 0 :contentReference[oaicite:64]{index=64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    pub order: serde_json::Value,
}

/// DELETE /portfolio/orders/{order_id} supports optional query parameter subaccount :contentReference[oaicite:65]{index=65}
#[derive(Debug, Clone, Default, Serialize)]
pub struct CancelOrderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    pub order: serde_json::Value,
    pub reduced_by: i64,
    pub reduced_by_fp: String,
}

