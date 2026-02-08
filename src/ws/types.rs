use crate::error::KalshiError;
use crate::rest::types::{EventPosition, MarketPosition};
use crate::types::{BuySell, FixedPointCount, FixedPointDollars, TradeTakerSide, YesNo};

use bytes::Bytes;
use serde::de::{Error as _, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsChannel {
    // Public (no auth required)
    Ticker,
    TickerV2,
    Trade,
    MarketLifecycleV2,
    Multivariate,

    // Private (auth required)
    OrderbookDelta,
    Fill,
    MarketPositions,
    Communications,
    OrderGroupUpdates,
}

impl WsChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            WsChannel::Ticker => "ticker",
            WsChannel::TickerV2 => "ticker_v2",
            WsChannel::Trade => "trade",
            WsChannel::MarketLifecycleV2 => "market_lifecycle_v2",
            WsChannel::Multivariate => "multivariate",
            WsChannel::OrderbookDelta => "orderbook_delta",
            WsChannel::Fill => "fill",
            WsChannel::MarketPositions => "market_positions",
            WsChannel::Communications => "communications",
            WsChannel::OrderGroupUpdates => "order_group_updates",
        }
    }

    pub fn is_private(self) -> bool {
        matches!(
            self,
            WsChannel::OrderbookDelta
                | WsChannel::Fill
                | WsChannel::MarketPositions
                | WsChannel::Communications
                | WsChannel::OrderGroupUpdates
        )
    }
}

impl fmt::Display for WsChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WsMsgType {
    Subscribed,
    Unsubscribed,
    Ok,
    ListSubscriptions,
    Error,
    Ticker,
    TickerV2,
    Trade,
    OrderbookSnapshot,
    OrderbookDelta,
    Fill,
    MarketPositions,
    MarketLifecycleV2,
    EventLifecycle,
    Multivariate,
    MultivariateLookup,
    Communications,
    RfqCreated,
    RfqDeleted,
    QuoteCreated,
    QuoteAccepted,
    QuoteExecuted,
    OrderGroupUpdates,
    Unknown(String),
}

impl WsMsgType {
    pub fn as_str(&self) -> &str {
        match self {
            WsMsgType::Subscribed => "subscribed",
            WsMsgType::Unsubscribed => "unsubscribed",
            WsMsgType::Ok => "ok",
            WsMsgType::ListSubscriptions => "list_subscriptions",
            WsMsgType::Error => "error",
            WsMsgType::Ticker => "ticker",
            WsMsgType::TickerV2 => "ticker_v2",
            WsMsgType::Trade => "trade",
            WsMsgType::OrderbookSnapshot => "orderbook_snapshot",
            WsMsgType::OrderbookDelta => "orderbook_delta",
            WsMsgType::Fill => "fill",
            WsMsgType::MarketPositions => "market_positions",
            WsMsgType::MarketLifecycleV2 => "market_lifecycle_v2",
            WsMsgType::EventLifecycle => "event_lifecycle",
            WsMsgType::Multivariate => "multivariate",
            WsMsgType::MultivariateLookup => "multivariate_lookup",
            WsMsgType::Communications => "communications",
            WsMsgType::RfqCreated => "rfq_created",
            WsMsgType::RfqDeleted => "rfq_deleted",
            WsMsgType::QuoteCreated => "quote_created",
            WsMsgType::QuoteAccepted => "quote_accepted",
            WsMsgType::QuoteExecuted => "quote_executed",
            WsMsgType::OrderGroupUpdates => "order_group_updates",
            WsMsgType::Unknown(value) => value.as_str(),
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "subscribed" => WsMsgType::Subscribed,
            "unsubscribed" => WsMsgType::Unsubscribed,
            "ok" => WsMsgType::Ok,
            "list_subscriptions" => WsMsgType::ListSubscriptions,
            "error" => WsMsgType::Error,
            "ticker" => WsMsgType::Ticker,
            "ticker_v2" => WsMsgType::TickerV2,
            "trade" => WsMsgType::Trade,
            "orderbook_snapshot" => WsMsgType::OrderbookSnapshot,
            "orderbook_delta" => WsMsgType::OrderbookDelta,
            "fill" => WsMsgType::Fill,
            "market_positions" => WsMsgType::MarketPositions,
            "market_lifecycle_v2" => WsMsgType::MarketLifecycleV2,
            "event_lifecycle" | "event_lifecycle_v2" => WsMsgType::EventLifecycle,
            "multivariate" => WsMsgType::Multivariate,
            "multivariate_lookup" => WsMsgType::MultivariateLookup,
            "communications" => WsMsgType::Communications,
            "rfq_created" => WsMsgType::RfqCreated,
            "rfq_deleted" => WsMsgType::RfqDeleted,
            "quote_created" => WsMsgType::QuoteCreated,
            "quote_accepted" => WsMsgType::QuoteAccepted,
            "quote_executed" => WsMsgType::QuoteExecuted,
            "order_group_updates" => WsMsgType::OrderGroupUpdates,
            _ => return None,
        })
    }

    fn from_string(value: String) -> Self {
        match value.as_str() {
            "subscribed" => WsMsgType::Subscribed,
            "unsubscribed" => WsMsgType::Unsubscribed,
            "ok" => WsMsgType::Ok,
            "list_subscriptions" => WsMsgType::ListSubscriptions,
            "error" => WsMsgType::Error,
            "ticker" => WsMsgType::Ticker,
            "ticker_v2" => WsMsgType::TickerV2,
            "trade" => WsMsgType::Trade,
            "orderbook_snapshot" => WsMsgType::OrderbookSnapshot,
            "orderbook_delta" => WsMsgType::OrderbookDelta,
            "fill" => WsMsgType::Fill,
            "market_positions" => WsMsgType::MarketPositions,
            "market_lifecycle_v2" => WsMsgType::MarketLifecycleV2,
            "event_lifecycle" | "event_lifecycle_v2" => WsMsgType::EventLifecycle,
            "multivariate" => WsMsgType::Multivariate,
            "multivariate_lookup" => WsMsgType::MultivariateLookup,
            "communications" => WsMsgType::Communications,
            "rfq_created" => WsMsgType::RfqCreated,
            "rfq_deleted" => WsMsgType::RfqDeleted,
            "quote_created" => WsMsgType::QuoteCreated,
            "quote_accepted" => WsMsgType::QuoteAccepted,
            "quote_executed" => WsMsgType::QuoteExecuted,
            "order_group_updates" => WsMsgType::OrderGroupUpdates,
            _ => WsMsgType::Unknown(value),
        }
    }
}

impl fmt::Display for WsMsgType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for WsMsgType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for WsMsgType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WsMsgTypeVisitor;

        impl<'de> Visitor<'de> for WsMsgTypeVisitor {
            type Value = WsMsgType;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a websocket message type string")
            }

            fn visit_borrowed_str<E: serde::de::Error>(
                self,
                value: &'de str,
            ) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_str(value)
                    .unwrap_or_else(|| WsMsgType::Unknown(value.to_owned())))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_str(value)
                    .unwrap_or_else(|| WsMsgType::Unknown(value.to_owned())))
            }

            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_string(value))
            }
        }

        deserializer.deserialize_str(WsMsgTypeVisitor)
    }
}

/// Subscription parameters for WebSocket channels.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WsSubscriptionParams {
    pub channels: Vec<WsChannel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_tickers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_initial_snapshot: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_factor: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_key: Option<String>,
}

impl WsSubscriptionParams {
    pub fn normalized(mut self) -> Self {
        self.channels.sort_by_key(|c| c.as_str());
        if let Some(ref mut tickers) = self.market_tickers {
            tickers.sort();
        }
        if let Some(ref mut ids) = self.market_ids {
            ids.sort();
        }
        if let Some(ref mut tickers) = self.event_tickers {
            tickers.sort();
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsSubscriptionInfo {
    pub sid: u64,
    pub channels: Vec<WsChannel>,
    #[serde(default)]
    pub market_tickers: Option<Vec<String>>,
    #[serde(default)]
    pub market_ids: Option<Vec<String>>,
    #[serde(default)]
    pub event_tickers: Option<Vec<String>>,
    #[serde(default)]
    pub send_initial_snapshot: Option<bool>,
    #[serde(default)]
    pub shard_factor: Option<u32>,
    #[serde(default)]
    pub shard_key: Option<String>,
}

/// Ticker channel message (type: "ticker")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTicker {
    pub market_ticker: String,
    pub market_id: String,
    pub price: i64,
    pub yes_bid: i64,
    pub yes_ask: i64,
    pub price_dollars: String,
    pub yes_bid_dollars: String,
    pub yes_ask_dollars: String,
    pub volume: i64,
    pub volume_fp: String,
    pub open_interest: i64,
    pub open_interest_fp: String,
    pub dollar_volume: i64,
    pub dollar_open_interest: i64,
    pub ts: i64,
}

/// Ticker V2 channel message (type: "ticker_v2")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTickerV2 {
    pub market_ticker: String,
    #[serde(default)]
    pub market_id: Option<String>,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default)]
    pub price_dollars: Option<String>,
    #[serde(default)]
    pub yes_bid: Option<i64>,
    #[serde(default)]
    pub yes_ask: Option<i64>,
    #[serde(default)]
    pub no_bid: Option<i64>,
    #[serde(default)]
    pub no_ask: Option<i64>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default)]
    pub volume_fp: Option<String>,
    #[serde(default)]
    pub open_interest: Option<i64>,
    #[serde(default)]
    pub open_interest_fp: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
}

/// Trade channel message (type: "trade")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTrade {
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
    pub yes_price_dollars: Option<String>,
    #[serde(default)]
    pub no_price_dollars: Option<String>,
    #[serde(default)]
    pub taker_side: Option<TradeTakerSide>,
    #[serde(default)]
    pub created_time: Option<String>,
}

/// Orderbook snapshot message (type: "orderbook_snapshot")
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderbookSnapshot {
    pub market_ticker: String,
    pub market_id: String,
    /// Price levels: (price_cents, quantity)
    #[serde(default)]
    pub yes: Vec<(i64, i64)>,
    /// Price levels: (price_cents, quantity)
    #[serde(default)]
    pub no: Vec<(i64, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default)]
    pub yes_dollars: Vec<(String, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default)]
    pub no_dollars: Vec<(String, i64)>,
    /// Price levels: (price_dollars, quantity_fp) - fully fixed-point
    #[serde(default)]
    pub yes_dollars_fp: Vec<(String, String)>,
    /// Price levels: (price_dollars, quantity_fp) - fully fixed-point
    #[serde(default)]
    pub no_dollars_fp: Vec<(String, String)>,
}

/// Orderbook delta message (type: "orderbook_delta")
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderbookDelta {
    pub market_ticker: String,
    pub market_id: String,
    pub price: i64,
    pub price_dollars: String,
    pub delta: i64,
    pub delta_fp: String,
    pub side: YesNo,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub subaccount: Option<i64>,
    #[serde(default)]
    pub ts: Option<String>,
}

/// Fill channel message (type: "fill")
#[derive(Debug, Clone, Deserialize)]
pub struct WsFill {
    pub fill_id: String,
    pub trade_id: String,
    pub order_id: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub ticker: String,
    pub market_ticker: String,
    pub side: YesNo,
    pub action: BuySell,
    pub count: i64,
    pub count_fp: String,
    pub yes_price: i64,
    pub no_price: i64,
    #[serde(alias = "yes_price_dollars")]
    pub yes_price_fixed: String,
    #[serde(alias = "no_price_dollars")]
    pub no_price_fixed: String,
    pub is_taker: bool,
    pub fee_cost: String,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub subaccount_number: Option<i64>,
    #[serde(default)]
    pub ts: Option<i64>,
}

/// Market lifecycle message (type: "market_lifecycle_v2")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleV2 {
    pub market_ticker: String,
    #[serde(default)]
    pub event_type: Option<WsMarketLifecycleEventType>,
    #[serde(default)]
    pub open_ts: Option<i64>,
    #[serde(default)]
    pub close_ts: Option<i64>,
    #[serde(default)]
    pub additional_metadata: Option<WsMarketLifecycleAdditionalMetadata>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WsMarketLifecycleEventType {
    Created,
    Activated,
    Deactivated,
    CloseDateUpdated,
    Determined,
    Settled,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleAdditionalMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub no_sub_title: Option<String>,
    #[serde(default)]
    pub rules_primary: Option<String>,
    #[serde(default)]
    pub rules_secondary: Option<String>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub expected_expiration_ts: Option<i64>,
    #[serde(default)]
    pub strike_type: Option<String>,
    #[serde(default)]
    pub floor_strike: Option<i64>,
    #[serde(default)]
    pub custom_strike: Option<BTreeMap<String, String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// Event lifecycle message (type: "event_lifecycle")
#[derive(Debug, Clone, Deserialize)]
pub struct WsEventLifecycleV2 {
    pub event_ticker: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub collateral_return_type: Option<String>,
    #[serde(default)]
    pub series_ticker: Option<String>,
    #[serde(default)]
    pub additional_metadata: Option<WsEventLifecycleAdditionalMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsEventLifecycleAdditionalMetadata {
    #[serde(default)]
    pub custom_strike: Option<BTreeMap<String, String>>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// Market positions message (type: "market_positions")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketPositions {
    #[serde(default)]
    pub market_positions: Vec<MarketPosition>,
    #[serde(default)]
    pub event_positions: Vec<EventPosition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMultivariateSelectedMarket {
    pub event_ticker: String,
    pub market_ticker: String,
    pub side: YesNo,
}

/// Multivariate message payload (type: "multivariate_lookup")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMultivariate {
    pub collection_ticker: String,
    pub event_ticker: String,
    pub market_ticker: String,
    pub selected_markets: Vec<WsMultivariateSelectedMarket>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsOrderGroupEventType {
    Created,
    Triggered,
    Reset,
    Deleted,
    LimitUpdated,
    #[serde(other)]
    Unknown,
}

/// Order group update message payload (type: "order_group_updates")
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderGroupUpdate {
    pub event_type: WsOrderGroupEventType,
    pub order_group_id: String,
    #[serde(default)]
    pub contracts_limit_fp: Option<FixedPointCount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMveSelectedLeg {
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub market_ticker: Option<String>,
    #[serde(default)]
    pub side: Option<YesNo>,
    #[serde(default)]
    pub yes_settlement_value_dollars: Option<FixedPointDollars>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsRfqCreated {
    pub id: String,
    pub creator_id: String,
    pub market_ticker: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub contracts: Option<i64>,
    #[serde(default)]
    pub contracts_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub target_cost: Option<i64>,
    #[serde(default)]
    pub target_cost_dollars: Option<FixedPointDollars>,
    pub created_ts: String,
    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<WsMveSelectedLeg>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsRfqDeleted {
    pub id: String,
    pub creator_id: String,
    pub market_ticker: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub contracts: Option<i64>,
    #[serde(default)]
    pub contracts_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub target_cost: Option<i64>,
    #[serde(default)]
    pub target_cost_dollars: Option<FixedPointDollars>,
    pub deleted_ts: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteCreated {
    pub quote_id: String,
    pub rfq_id: String,
    pub quote_creator_id: String,
    pub market_ticker: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    pub yes_bid: i64,
    pub no_bid: i64,
    pub yes_bid_dollars: FixedPointDollars,
    pub no_bid_dollars: FixedPointDollars,
    #[serde(default)]
    pub yes_contracts_offered: Option<i64>,
    #[serde(default)]
    pub no_contracts_offered: Option<i64>,
    #[serde(default)]
    pub yes_contracts_offered_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub no_contracts_offered_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub rfq_target_cost: Option<i64>,
    #[serde(default)]
    pub rfq_target_cost_dollars: Option<FixedPointDollars>,
    pub created_ts: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteAccepted {
    pub quote_id: String,
    pub rfq_id: String,
    pub quote_creator_id: String,
    pub market_ticker: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    pub yes_bid: i64,
    pub no_bid: i64,
    pub yes_bid_dollars: FixedPointDollars,
    pub no_bid_dollars: FixedPointDollars,
    #[serde(default)]
    pub accepted_side: Option<YesNo>,
    #[serde(default)]
    pub contracts_accepted: Option<i64>,
    #[serde(default)]
    pub yes_contracts_offered: Option<i64>,
    #[serde(default)]
    pub no_contracts_offered: Option<i64>,
    #[serde(default)]
    pub contracts_accepted_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub yes_contracts_offered_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub no_contracts_offered_fp: Option<FixedPointCount>,
    #[serde(default)]
    pub rfq_target_cost: Option<i64>,
    #[serde(default)]
    pub rfq_target_cost_dollars: Option<FixedPointDollars>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteExecuted {
    pub quote_id: String,
    pub rfq_id: String,
    pub quote_creator_id: String,
    pub rfq_creator_id: String,
    pub order_id: String,
    pub client_order_id: String,
    pub market_ticker: String,
    pub executed_ts: String,
}

/// Communications message payloads (RFQs and quotes).
#[derive(Debug, Clone)]
pub enum WsCommunications {
    RfqCreated(WsRfqCreated),
    RfqDeleted(WsRfqDeleted),
    QuoteCreated(WsQuoteCreated),
    QuoteAccepted(WsQuoteAccepted),
    QuoteExecuted(WsQuoteExecuted),
}

/// Borrowed fixed-point dollar string (e.g. "0.5600").
pub type FixedPointDollarsRef<'a> = Cow<'a, str>;

/// Borrowed fixed-point contract count string (e.g. "10.00").
pub type FixedPointCountRef<'a> = Cow<'a, str>;

#[derive(Debug, Clone, Deserialize)]
pub struct MarketPositionRef<'a> {
    #[serde(borrow)]
    pub ticker: Cow<'a, str>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default, borrow)]
    pub position_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub fees_paid: Option<i64>,
    #[serde(default, borrow)]
    pub fees_paid_fp: Option<FixedPointDollarsRef<'a>>,
    #[serde(default)]
    pub resting_orders: Option<i64>,
    #[serde(default, borrow)]
    pub resting_orders_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub total_traded: Option<i64>,
    #[serde(default, borrow)]
    pub total_traded_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub subaccount: Option<u32>,
}

impl<'a> MarketPositionRef<'a> {
    pub fn into_owned(self) -> MarketPosition {
        MarketPosition {
            ticker: self.ticker.into_owned(),
            position: self.position,
            position_fp: self.position_fp.map(Cow::into_owned),
            fees_paid: self.fees_paid,
            fees_paid_fp: self.fees_paid_fp.map(Cow::into_owned),
            resting_orders: self.resting_orders,
            resting_orders_fp: self.resting_orders_fp.map(Cow::into_owned),
            total_traded: self.total_traded,
            total_traded_fp: self.total_traded_fp.map(Cow::into_owned),
            subaccount: self.subaccount,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventPositionRef<'a> {
    #[serde(borrow)]
    pub event_ticker: Cow<'a, str>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default, borrow)]
    pub position_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub fees_paid: Option<i64>,
    #[serde(default, borrow)]
    pub fees_paid_fp: Option<FixedPointDollarsRef<'a>>,
    #[serde(default)]
    pub resting_orders: Option<i64>,
    #[serde(default, borrow)]
    pub resting_orders_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub total_traded: Option<i64>,
    #[serde(default, borrow)]
    pub total_traded_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub subaccount: Option<u32>,
}

impl<'a> EventPositionRef<'a> {
    pub fn into_owned(self) -> EventPosition {
        EventPosition {
            event_ticker: self.event_ticker.into_owned(),
            position: self.position,
            position_fp: self.position_fp.map(Cow::into_owned),
            fees_paid: self.fees_paid,
            fees_paid_fp: self.fees_paid_fp.map(Cow::into_owned),
            resting_orders: self.resting_orders,
            resting_orders_fp: self.resting_orders_fp.map(Cow::into_owned),
            total_traded: self.total_traded,
            total_traded_fp: self.total_traded_fp.map(Cow::into_owned),
            subaccount: self.subaccount,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsSubscriptionInfoRef<'a> {
    pub sid: u64,
    pub channels: Vec<WsChannel>,
    #[serde(default, borrow)]
    pub market_tickers: Option<Vec<Cow<'a, str>>>,
    #[serde(default, borrow)]
    pub market_ids: Option<Vec<Cow<'a, str>>>,
    #[serde(default, borrow)]
    pub event_tickers: Option<Vec<Cow<'a, str>>>,
    #[serde(default)]
    pub send_initial_snapshot: Option<bool>,
    #[serde(default)]
    pub shard_factor: Option<u32>,
    #[serde(default, borrow)]
    pub shard_key: Option<Cow<'a, str>>,
}

impl<'a> WsSubscriptionInfoRef<'a> {
    pub fn into_owned(self) -> WsSubscriptionInfo {
        WsSubscriptionInfo {
            sid: self.sid,
            channels: self.channels,
            market_tickers: self
                .market_tickers
                .map(|v| v.into_iter().map(Cow::into_owned).collect()),
            market_ids: self
                .market_ids
                .map(|v| v.into_iter().map(Cow::into_owned).collect()),
            event_tickers: self
                .event_tickers
                .map(|v| v.into_iter().map(Cow::into_owned).collect()),
            send_initial_snapshot: self.send_initial_snapshot,
            shard_factor: self.shard_factor,
            shard_key: self.shard_key.map(Cow::into_owned),
        }
    }
}

/// Ticker channel message (type: "ticker")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTickerRef<'a> {
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub market_id: Cow<'a, str>,
    pub price: i64,
    pub yes_bid: i64,
    pub yes_ask: i64,
    #[serde(borrow)]
    pub price_dollars: Cow<'a, str>,
    #[serde(borrow)]
    pub yes_bid_dollars: Cow<'a, str>,
    #[serde(borrow)]
    pub yes_ask_dollars: Cow<'a, str>,
    pub volume: i64,
    #[serde(borrow)]
    pub volume_fp: Cow<'a, str>,
    pub open_interest: i64,
    #[serde(borrow)]
    pub open_interest_fp: Cow<'a, str>,
    pub dollar_volume: i64,
    pub dollar_open_interest: i64,
    pub ts: i64,
}

impl<'a> WsTickerRef<'a> {
    pub fn into_owned(self) -> WsTicker {
        WsTicker {
            market_ticker: self.market_ticker.into_owned(),
            market_id: self.market_id.into_owned(),
            price: self.price,
            yes_bid: self.yes_bid,
            yes_ask: self.yes_ask,
            price_dollars: self.price_dollars.into_owned(),
            yes_bid_dollars: self.yes_bid_dollars.into_owned(),
            yes_ask_dollars: self.yes_ask_dollars.into_owned(),
            volume: self.volume,
            volume_fp: self.volume_fp.into_owned(),
            open_interest: self.open_interest,
            open_interest_fp: self.open_interest_fp.into_owned(),
            dollar_volume: self.dollar_volume,
            dollar_open_interest: self.dollar_open_interest,
            ts: self.ts,
        }
    }
}

/// Ticker V2 channel message (type: "ticker_v2")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTickerV2Ref<'a> {
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(default, borrow)]
    pub market_id: Option<Cow<'a, str>>,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default, borrow)]
    pub price_dollars: Option<Cow<'a, str>>,
    #[serde(default)]
    pub yes_bid: Option<i64>,
    #[serde(default)]
    pub yes_ask: Option<i64>,
    #[serde(default)]
    pub no_bid: Option<i64>,
    #[serde(default)]
    pub no_ask: Option<i64>,
    #[serde(default)]
    pub volume: Option<i64>,
    #[serde(default, borrow)]
    pub volume_fp: Option<Cow<'a, str>>,
    #[serde(default)]
    pub open_interest: Option<i64>,
    #[serde(default, borrow)]
    pub open_interest_fp: Option<Cow<'a, str>>,
    #[serde(default)]
    pub ts: Option<i64>,
}

impl<'a> WsTickerV2Ref<'a> {
    pub fn into_owned(self) -> WsTickerV2 {
        WsTickerV2 {
            market_ticker: self.market_ticker.into_owned(),
            market_id: self.market_id.map(Cow::into_owned),
            price: self.price,
            price_dollars: self.price_dollars.map(Cow::into_owned),
            yes_bid: self.yes_bid,
            yes_ask: self.yes_ask,
            no_bid: self.no_bid,
            no_ask: self.no_ask,
            volume: self.volume,
            volume_fp: self.volume_fp.map(Cow::into_owned),
            open_interest: self.open_interest,
            open_interest_fp: self.open_interest_fp.map(Cow::into_owned),
            ts: self.ts,
        }
    }
}

/// Trade channel message (type: "trade")
#[derive(Debug, Clone, Deserialize)]
pub struct WsTradeRef<'a> {
    #[serde(borrow)]
    pub trade_id: Cow<'a, str>,
    #[serde(borrow)]
    pub ticker: Cow<'a, str>,
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default, borrow)]
    pub count_fp: Option<Cow<'a, str>>,
    #[serde(default)]
    pub yes_price: Option<i64>,
    #[serde(default)]
    pub no_price: Option<i64>,
    #[serde(default, borrow)]
    pub yes_price_dollars: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub no_price_dollars: Option<Cow<'a, str>>,
    #[serde(default)]
    pub taker_side: Option<TradeTakerSide>,
    #[serde(default, borrow)]
    pub created_time: Option<Cow<'a, str>>,
}

impl<'a> WsTradeRef<'a> {
    pub fn into_owned(self) -> WsTrade {
        WsTrade {
            trade_id: self.trade_id.into_owned(),
            ticker: self.ticker.into_owned(),
            price: self.price,
            count: self.count,
            count_fp: self.count_fp.map(Cow::into_owned),
            yes_price: self.yes_price,
            no_price: self.no_price,
            yes_price_dollars: self.yes_price_dollars.map(Cow::into_owned),
            no_price_dollars: self.no_price_dollars.map(Cow::into_owned),
            taker_side: self.taker_side,
            created_time: self.created_time.map(Cow::into_owned),
        }
    }
}

/// Orderbook snapshot message (type: "orderbook_snapshot")
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderbookSnapshotRef<'a> {
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub market_id: Cow<'a, str>,
    /// Price levels: (price_cents, quantity)
    #[serde(default)]
    pub yes: Vec<(i64, i64)>,
    /// Price levels: (price_cents, quantity)
    #[serde(default)]
    pub no: Vec<(i64, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default, borrow)]
    pub yes_dollars: Vec<(Cow<'a, str>, i64)>,
    /// Price levels: (price_dollars, quantity)
    #[serde(default, borrow)]
    pub no_dollars: Vec<(Cow<'a, str>, i64)>,
    /// Price levels: (price_dollars, quantity_fp) - fully fixed-point
    #[serde(default, borrow)]
    pub yes_dollars_fp: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    /// Price levels: (price_dollars, quantity_fp) - fully fixed-point
    #[serde(default, borrow)]
    pub no_dollars_fp: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> WsOrderbookSnapshotRef<'a> {
    pub fn into_owned(self) -> WsOrderbookSnapshot {
        WsOrderbookSnapshot {
            market_ticker: self.market_ticker.into_owned(),
            market_id: self.market_id.into_owned(),
            yes: self.yes,
            no: self.no,
            yes_dollars: self
                .yes_dollars
                .into_iter()
                .map(|(p, q)| (p.into_owned(), q))
                .collect(),
            no_dollars: self
                .no_dollars
                .into_iter()
                .map(|(p, q)| (p.into_owned(), q))
                .collect(),
            yes_dollars_fp: self
                .yes_dollars_fp
                .into_iter()
                .map(|(p, q)| (p.into_owned(), q.into_owned()))
                .collect(),
            no_dollars_fp: self
                .no_dollars_fp
                .into_iter()
                .map(|(p, q)| (p.into_owned(), q.into_owned()))
                .collect(),
        }
    }
}

/// Orderbook delta message (type: "orderbook_delta")
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderbookDeltaRef<'a> {
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub market_id: Cow<'a, str>,
    pub price: i64,
    #[serde(borrow)]
    pub price_dollars: Cow<'a, str>,
    pub delta: i64,
    #[serde(borrow)]
    pub delta_fp: Cow<'a, str>,
    pub side: YesNo,
    #[serde(default, borrow)]
    pub client_order_id: Option<Cow<'a, str>>,
    #[serde(default)]
    pub subaccount: Option<i64>,
    #[serde(default, borrow)]
    pub ts: Option<Cow<'a, str>>,
}

impl<'a> WsOrderbookDeltaRef<'a> {
    pub fn into_owned(self) -> WsOrderbookDelta {
        WsOrderbookDelta {
            market_ticker: self.market_ticker.into_owned(),
            market_id: self.market_id.into_owned(),
            price: self.price,
            price_dollars: self.price_dollars.into_owned(),
            delta: self.delta,
            delta_fp: self.delta_fp.into_owned(),
            side: self.side,
            client_order_id: self.client_order_id.map(Cow::into_owned),
            subaccount: self.subaccount,
            ts: self.ts.map(Cow::into_owned),
        }
    }
}

/// Fill channel message (type: "fill")
#[derive(Debug, Clone, Deserialize)]
pub struct WsFillRef<'a> {
    #[serde(borrow)]
    pub fill_id: Cow<'a, str>,
    #[serde(borrow)]
    pub trade_id: Cow<'a, str>,
    #[serde(borrow)]
    pub order_id: Cow<'a, str>,
    #[serde(default, borrow)]
    pub client_order_id: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    pub side: YesNo,
    pub action: BuySell,
    pub count: i64,
    #[serde(borrow)]
    pub count_fp: Cow<'a, str>,
    pub yes_price: i64,
    pub no_price: i64,
    #[serde(alias = "yes_price_dollars", borrow)]
    pub yes_price_fixed: Cow<'a, str>,
    #[serde(alias = "no_price_dollars", borrow)]
    pub no_price_fixed: Cow<'a, str>,
    pub is_taker: bool,
    #[serde(borrow)]
    pub fee_cost: Cow<'a, str>,
    #[serde(default, borrow)]
    pub created_time: Option<Cow<'a, str>>,
    #[serde(default)]
    pub subaccount_number: Option<i64>,
    #[serde(default)]
    pub ts: Option<i64>,
}

impl<'a> WsFillRef<'a> {
    pub fn into_owned(self) -> WsFill {
        WsFill {
            fill_id: self.fill_id.into_owned(),
            trade_id: self.trade_id.into_owned(),
            order_id: self.order_id.into_owned(),
            client_order_id: self.client_order_id.map(Cow::into_owned),
            ticker: self.ticker.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            side: self.side,
            action: self.action,
            count: self.count,
            count_fp: self.count_fp.into_owned(),
            yes_price: self.yes_price,
            no_price: self.no_price,
            yes_price_fixed: self.yes_price_fixed.into_owned(),
            no_price_fixed: self.no_price_fixed.into_owned(),
            is_taker: self.is_taker,
            fee_cost: self.fee_cost.into_owned(),
            created_time: self.created_time.map(Cow::into_owned),
            subaccount_number: self.subaccount_number,
            ts: self.ts,
        }
    }
}

/// Market lifecycle message (type: "market_lifecycle_v2")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketLifecycleV2Ref<'a> {
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    /// market status
    #[serde(default)]
    pub status: Option<MarketStatus>,
    #[serde(default)]
    pub can_trade: Option<bool>,
    #[serde(default)]
    pub can_settle: Option<bool>,
    #[serde(default, borrow)]
    pub open_time: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub close_time: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub settled_time: Option<Cow<'a, str>>,
}

impl<'a> WsMarketLifecycleV2Ref<'a> {
    pub fn into_owned(self) -> WsMarketLifecycleV2 {
        WsMarketLifecycleV2 {
            market_ticker: self.market_ticker.into_owned(),
            status: self.status,
            can_trade: self.can_trade,
            can_settle: self.can_settle,
            open_time: self.open_time.map(Cow::into_owned),
            close_time: self.close_time.map(Cow::into_owned),
            settled_time: self.settled_time.map(Cow::into_owned),
        }
    }
}

/// Market positions message (type: "market_positions")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMarketPositionsRef<'a> {
    #[serde(default, borrow)]
    pub market_positions: Vec<MarketPositionRef<'a>>,
    #[serde(default, borrow)]
    pub event_positions: Vec<EventPositionRef<'a>>,
}

impl<'a> WsMarketPositionsRef<'a> {
    pub fn into_owned(self) -> WsMarketPositions {
        WsMarketPositions {
            market_positions: self
                .market_positions
                .into_iter()
                .map(MarketPositionRef::into_owned)
                .collect(),
            event_positions: self
                .event_positions
                .into_iter()
                .map(EventPositionRef::into_owned)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMultivariateSelectedMarketRef<'a> {
    #[serde(borrow)]
    pub event_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    pub side: YesNo,
}

impl<'a> WsMultivariateSelectedMarketRef<'a> {
    pub fn into_owned(self) -> WsMultivariateSelectedMarket {
        WsMultivariateSelectedMarket {
            event_ticker: self.event_ticker.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            side: self.side,
        }
    }
}

/// Multivariate message payload (type: "multivariate_lookup")
#[derive(Debug, Clone, Deserialize)]
pub struct WsMultivariateRef<'a> {
    #[serde(borrow)]
    pub collection_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub event_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub selected_markets: Vec<WsMultivariateSelectedMarketRef<'a>>,
}

impl<'a> WsMultivariateRef<'a> {
    pub fn into_owned(self) -> WsMultivariate {
        WsMultivariate {
            collection_ticker: self.collection_ticker.into_owned(),
            event_ticker: self.event_ticker.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            selected_markets: self
                .selected_markets
                .into_iter()
                .map(WsMultivariateSelectedMarketRef::into_owned)
                .collect(),
        }
    }
}

/// Order group update message payload (type: "order_group_updates")
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrderGroupUpdateRef<'a> {
    pub event_type: WsOrderGroupEventType,
    #[serde(borrow)]
    pub order_group_id: Cow<'a, str>,
    #[serde(default, borrow)]
    pub contracts_limit_fp: Option<FixedPointCountRef<'a>>,
}

impl<'a> WsOrderGroupUpdateRef<'a> {
    pub fn into_owned(self) -> WsOrderGroupUpdate {
        WsOrderGroupUpdate {
            event_type: self.event_type,
            order_group_id: self.order_group_id.into_owned(),
            contracts_limit_fp: self.contracts_limit_fp.map(Cow::into_owned),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsMveSelectedLegRef<'a> {
    #[serde(default, borrow)]
    pub event_ticker: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub market_ticker: Option<Cow<'a, str>>,
    #[serde(default)]
    pub side: Option<YesNo>,
    #[serde(default, borrow)]
    pub yes_settlement_value_dollars: Option<FixedPointDollarsRef<'a>>,
}

impl<'a> WsMveSelectedLegRef<'a> {
    pub fn into_owned(self) -> WsMveSelectedLeg {
        WsMveSelectedLeg {
            event_ticker: self.event_ticker.map(Cow::into_owned),
            market_ticker: self.market_ticker.map(Cow::into_owned),
            side: self.side,
            yes_settlement_value_dollars: self.yes_settlement_value_dollars.map(Cow::into_owned),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsRfqCreatedRef<'a> {
    #[serde(borrow)]
    pub id: Cow<'a, str>,
    #[serde(borrow)]
    pub creator_id: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(default, borrow)]
    pub event_ticker: Option<Cow<'a, str>>,
    #[serde(default)]
    pub contracts: Option<i64>,
    #[serde(default, borrow)]
    pub contracts_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub target_cost: Option<i64>,
    #[serde(default, borrow)]
    pub target_cost_dollars: Option<FixedPointDollarsRef<'a>>,
    #[serde(borrow)]
    pub created_ts: Cow<'a, str>,
    #[serde(default, borrow)]
    pub mve_collection_ticker: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub mve_selected_legs: Option<Vec<WsMveSelectedLegRef<'a>>>,
}

impl<'a> WsRfqCreatedRef<'a> {
    pub fn into_owned(self) -> WsRfqCreated {
        WsRfqCreated {
            id: self.id.into_owned(),
            creator_id: self.creator_id.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            event_ticker: self.event_ticker.map(Cow::into_owned),
            contracts: self.contracts,
            contracts_fp: self.contracts_fp.map(Cow::into_owned),
            target_cost: self.target_cost,
            target_cost_dollars: self.target_cost_dollars.map(Cow::into_owned),
            created_ts: self.created_ts.into_owned(),
            mve_collection_ticker: self.mve_collection_ticker.map(Cow::into_owned),
            mve_selected_legs: self.mve_selected_legs.map(|legs| {
                legs.into_iter()
                    .map(WsMveSelectedLegRef::into_owned)
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsRfqDeletedRef<'a> {
    #[serde(borrow)]
    pub id: Cow<'a, str>,
    #[serde(borrow)]
    pub creator_id: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(default, borrow)]
    pub event_ticker: Option<Cow<'a, str>>,
    #[serde(default)]
    pub contracts: Option<i64>,
    #[serde(default, borrow)]
    pub contracts_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub target_cost: Option<i64>,
    #[serde(default, borrow)]
    pub target_cost_dollars: Option<FixedPointDollarsRef<'a>>,
    #[serde(borrow)]
    pub deleted_ts: Cow<'a, str>,
}

impl<'a> WsRfqDeletedRef<'a> {
    pub fn into_owned(self) -> WsRfqDeleted {
        WsRfqDeleted {
            id: self.id.into_owned(),
            creator_id: self.creator_id.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            event_ticker: self.event_ticker.map(Cow::into_owned),
            contracts: self.contracts,
            contracts_fp: self.contracts_fp.map(Cow::into_owned),
            target_cost: self.target_cost,
            target_cost_dollars: self.target_cost_dollars.map(Cow::into_owned),
            deleted_ts: self.deleted_ts.into_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteCreatedRef<'a> {
    #[serde(borrow)]
    pub quote_id: Cow<'a, str>,
    #[serde(borrow)]
    pub rfq_id: Cow<'a, str>,
    #[serde(borrow)]
    pub quote_creator_id: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(default, borrow)]
    pub event_ticker: Option<Cow<'a, str>>,
    pub yes_bid: i64,
    pub no_bid: i64,
    #[serde(borrow)]
    pub yes_bid_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub no_bid_dollars: FixedPointDollarsRef<'a>,
    #[serde(default)]
    pub yes_contracts_offered: Option<i64>,
    #[serde(default)]
    pub no_contracts_offered: Option<i64>,
    #[serde(default, borrow)]
    pub yes_contracts_offered_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default, borrow)]
    pub no_contracts_offered_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub rfq_target_cost: Option<i64>,
    #[serde(default, borrow)]
    pub rfq_target_cost_dollars: Option<FixedPointDollarsRef<'a>>,
    #[serde(borrow)]
    pub created_ts: Cow<'a, str>,
}

impl<'a> WsQuoteCreatedRef<'a> {
    pub fn into_owned(self) -> WsQuoteCreated {
        WsQuoteCreated {
            quote_id: self.quote_id.into_owned(),
            rfq_id: self.rfq_id.into_owned(),
            quote_creator_id: self.quote_creator_id.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            event_ticker: self.event_ticker.map(Cow::into_owned),
            yes_bid: self.yes_bid,
            no_bid: self.no_bid,
            yes_bid_dollars: self.yes_bid_dollars.into_owned(),
            no_bid_dollars: self.no_bid_dollars.into_owned(),
            yes_contracts_offered: self.yes_contracts_offered,
            no_contracts_offered: self.no_contracts_offered,
            yes_contracts_offered_fp: self.yes_contracts_offered_fp.map(Cow::into_owned),
            no_contracts_offered_fp: self.no_contracts_offered_fp.map(Cow::into_owned),
            rfq_target_cost: self.rfq_target_cost,
            rfq_target_cost_dollars: self.rfq_target_cost_dollars.map(Cow::into_owned),
            created_ts: self.created_ts.into_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteAcceptedRef<'a> {
    #[serde(borrow)]
    pub quote_id: Cow<'a, str>,
    #[serde(borrow)]
    pub rfq_id: Cow<'a, str>,
    #[serde(borrow)]
    pub quote_creator_id: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(default, borrow)]
    pub event_ticker: Option<Cow<'a, str>>,
    pub yes_bid: i64,
    pub no_bid: i64,
    #[serde(borrow)]
    pub yes_bid_dollars: FixedPointDollarsRef<'a>,
    #[serde(borrow)]
    pub no_bid_dollars: FixedPointDollarsRef<'a>,
    #[serde(default)]
    pub accepted_side: Option<YesNo>,
    #[serde(default)]
    pub contracts_accepted: Option<i64>,
    #[serde(default)]
    pub yes_contracts_offered: Option<i64>,
    #[serde(default)]
    pub no_contracts_offered: Option<i64>,
    #[serde(default, borrow)]
    pub contracts_accepted_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default, borrow)]
    pub yes_contracts_offered_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default, borrow)]
    pub no_contracts_offered_fp: Option<FixedPointCountRef<'a>>,
    #[serde(default)]
    pub rfq_target_cost: Option<i64>,
    #[serde(default, borrow)]
    pub rfq_target_cost_dollars: Option<FixedPointDollarsRef<'a>>,
}

impl<'a> WsQuoteAcceptedRef<'a> {
    pub fn into_owned(self) -> WsQuoteAccepted {
        WsQuoteAccepted {
            quote_id: self.quote_id.into_owned(),
            rfq_id: self.rfq_id.into_owned(),
            quote_creator_id: self.quote_creator_id.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            event_ticker: self.event_ticker.map(Cow::into_owned),
            yes_bid: self.yes_bid,
            no_bid: self.no_bid,
            yes_bid_dollars: self.yes_bid_dollars.into_owned(),
            no_bid_dollars: self.no_bid_dollars.into_owned(),
            accepted_side: self.accepted_side,
            contracts_accepted: self.contracts_accepted,
            yes_contracts_offered: self.yes_contracts_offered,
            no_contracts_offered: self.no_contracts_offered,
            contracts_accepted_fp: self.contracts_accepted_fp.map(Cow::into_owned),
            yes_contracts_offered_fp: self.yes_contracts_offered_fp.map(Cow::into_owned),
            no_contracts_offered_fp: self.no_contracts_offered_fp.map(Cow::into_owned),
            rfq_target_cost: self.rfq_target_cost,
            rfq_target_cost_dollars: self.rfq_target_cost_dollars.map(Cow::into_owned),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuoteExecutedRef<'a> {
    #[serde(borrow)]
    pub quote_id: Cow<'a, str>,
    #[serde(borrow)]
    pub rfq_id: Cow<'a, str>,
    #[serde(borrow)]
    pub quote_creator_id: Cow<'a, str>,
    #[serde(borrow)]
    pub rfq_creator_id: Cow<'a, str>,
    #[serde(borrow)]
    pub order_id: Cow<'a, str>,
    #[serde(borrow)]
    pub client_order_id: Cow<'a, str>,
    #[serde(borrow)]
    pub market_ticker: Cow<'a, str>,
    #[serde(borrow)]
    pub executed_ts: Cow<'a, str>,
}

impl<'a> WsQuoteExecutedRef<'a> {
    pub fn into_owned(self) -> WsQuoteExecuted {
        WsQuoteExecuted {
            quote_id: self.quote_id.into_owned(),
            rfq_id: self.rfq_id.into_owned(),
            quote_creator_id: self.quote_creator_id.into_owned(),
            rfq_creator_id: self.rfq_creator_id.into_owned(),
            order_id: self.order_id.into_owned(),
            client_order_id: self.client_order_id.into_owned(),
            market_ticker: self.market_ticker.into_owned(),
            executed_ts: self.executed_ts.into_owned(),
        }
    }
}

/// Communications message payloads (RFQs and quotes).
#[derive(Debug, Clone)]
pub enum WsCommunicationsRef<'a> {
    RfqCreated(WsRfqCreatedRef<'a>),
    RfqDeleted(WsRfqDeletedRef<'a>),
    QuoteCreated(WsQuoteCreatedRef<'a>),
    QuoteAccepted(WsQuoteAcceptedRef<'a>),
    QuoteExecuted(WsQuoteExecutedRef<'a>),
}

impl<'a> WsCommunicationsRef<'a> {
    pub fn into_owned(self) -> WsCommunications {
        match self {
            WsCommunicationsRef::RfqCreated(msg) => WsCommunications::RfqCreated(msg.into_owned()),
            WsCommunicationsRef::RfqDeleted(msg) => WsCommunications::RfqDeleted(msg.into_owned()),
            WsCommunicationsRef::QuoteCreated(msg) => {
                WsCommunications::QuoteCreated(msg.into_owned())
            }
            WsCommunicationsRef::QuoteAccepted(msg) => {
                WsCommunications::QuoteAccepted(msg.into_owned())
            }
            WsCommunicationsRef::QuoteExecuted(msg) => {
                WsCommunications::QuoteExecuted(msg.into_owned())
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsListSubscriptionsRef<'a> {
    #[serde(default, borrow)]
    pub subscriptions: Vec<WsSubscriptionInfoRef<'a>>,
}

impl<'a> WsListSubscriptionsRef<'a> {
    pub fn into_owned(self) -> WsListSubscriptions {
        WsListSubscriptions {
            subscriptions: self
                .subscriptions
                .into_iter()
                .map(WsSubscriptionInfoRef::into_owned)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsErrorRef<'a> {
    #[serde(default)]
    pub code: Option<i64>,
    #[serde(default, borrow)]
    pub message: Option<Cow<'a, str>>,
}

impl<'a> WsErrorRef<'a> {
    pub fn into_owned(self) -> WsError {
        WsError {
            code: self.code,
            message: self.message.map(Cow::into_owned),
        }
    }
}

/// Envelope used by Kalshi WS (data + errors use "type")
#[derive(Debug, Clone, Deserialize)]
pub struct WsEnvelope {
    pub id: Option<u64>,
    #[serde(rename = "type")]
    pub msg_type: WsMsgType,
    pub sid: Option<u64>,
    pub seq: Option<u64>,
    pub msg: Option<Box<RawValue>>,
    #[serde(default)]
    pub subscriptions: Option<Vec<WsSubscriptionInfo>>,
}

impl WsEnvelope {
    pub fn msg_raw(&self) -> Option<&str> {
        self.msg.as_deref().map(|raw| raw.get())
    }

    pub fn into_message(self) -> Result<WsMessage, KalshiError> {
        fn parse_msg<T: for<'de> Deserialize<'de>>(
            msg: &Option<Box<RawValue>>,
        ) -> Result<T, serde_json::Error> {
            let raw = msg
                .as_deref()
                .ok_or_else(|| serde_json::Error::custom("missing msg"))?;
            serde_json::from_str(raw.get())
        }

        let WsEnvelope {
            id,
            msg_type,
            sid,
            seq,
            msg,
            subscriptions,
        } = self;

        match msg_type {
            WsMsgType::Subscribed => Ok(WsMessage::Subscribed { id, sid }),
            WsMsgType::Unsubscribed => Ok(WsMessage::Unsubscribed { id, sid }),
            WsMsgType::Ok => Ok(WsMessage::Ok { id }),
            WsMsgType::ListSubscriptions => {
                let subs = if msg.is_some() {
                    let parsed: WsListSubscriptions = parse_msg(&msg)?;
                    parsed.subscriptions
                } else {
                    subscriptions.unwrap_or_default()
                };
                Ok(WsMessage::ListSubscriptions {
                    id,
                    subscriptions: subs,
                })
            }
            WsMsgType::Error => {
                let error = if msg.is_some() {
                    parse_msg(&msg)?
                } else {
                    WsError {
                        code: None,
                        message: None,
                    }
                };
                Ok(WsMessage::Error { id, error })
            }
            WsMsgType::Ticker => Ok(WsMessage::Data(WsDataMessage::Ticker {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::TickerV2 => Ok(WsMessage::Data(WsDataMessage::TickerV2 {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::Trade => Ok(WsMessage::Data(WsDataMessage::Trade {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::OrderbookSnapshot => Ok(WsMessage::Data(WsDataMessage::OrderbookSnapshot {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::OrderbookDelta => Ok(WsMessage::Data(WsDataMessage::OrderbookDelta {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::Fill => Ok(WsMessage::Data(WsDataMessage::Fill {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::MarketPositions => Ok(WsMessage::Data(WsDataMessage::MarketPositions {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::MarketLifecycleV2 => Ok(WsMessage::Data(WsDataMessage::MarketLifecycleV2 {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::EventLifecycle => Ok(WsMessage::Data(WsDataMessage::EventLifecycle {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::Multivariate | WsMsgType::MultivariateLookup => {
                Ok(WsMessage::Data(WsDataMessage::Multivariate {
                    sid,
                    seq,
                    msg: parse_msg(&msg)?,
                }))
            }
            WsMsgType::RfqCreated => Ok(WsMessage::Data(WsDataMessage::Communications {
                sid,
                seq,
                msg: WsCommunications::RfqCreated(parse_msg(&msg)?),
            })),
            WsMsgType::RfqDeleted => Ok(WsMessage::Data(WsDataMessage::Communications {
                sid,
                seq,
                msg: WsCommunications::RfqDeleted(parse_msg(&msg)?),
            })),
            WsMsgType::QuoteCreated => Ok(WsMessage::Data(WsDataMessage::Communications {
                sid,
                seq,
                msg: WsCommunications::QuoteCreated(parse_msg(&msg)?),
            })),
            WsMsgType::QuoteAccepted => Ok(WsMessage::Data(WsDataMessage::Communications {
                sid,
                seq,
                msg: WsCommunications::QuoteAccepted(parse_msg(&msg)?),
            })),
            WsMsgType::QuoteExecuted => Ok(WsMessage::Data(WsDataMessage::Communications {
                sid,
                seq,
                msg: WsCommunications::QuoteExecuted(parse_msg(&msg)?),
            })),
            WsMsgType::OrderGroupUpdates => Ok(WsMessage::Data(WsDataMessage::OrderGroupUpdates {
                sid,
                seq,
                msg: parse_msg(&msg)?,
            })),
            WsMsgType::Communications => Ok(WsMessage::Unknown {
                msg_type: WsMsgType::Communications,
                raw: msg,
            }),
            other => Ok(WsMessage::Unknown {
                msg_type: other,
                raw: msg,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsEnvelopeRef<'a> {
    pub id: Option<u64>,
    #[serde(rename = "type")]
    pub msg_type: WsMsgType,
    pub sid: Option<u64>,
    pub seq: Option<u64>,
    #[serde(borrow)]
    pub msg: Option<&'a RawValue>,
    #[serde(default, borrow)]
    pub subscriptions: Option<Vec<WsSubscriptionInfoRef<'a>>>,
}

fn parse_borrowed_msg<'a, T: Deserialize<'a>>(
    msg: Option<&'a RawValue>,
) -> Result<T, serde_json::Error> {
    let raw = msg.ok_or_else(|| serde_json::Error::custom("missing msg"))?;
    serde_json::from_str(raw.get())
}

impl<'a> WsEnvelopeRef<'a> {
    pub fn msg_raw(&self) -> Option<&str> {
        self.msg.as_deref().map(|raw| raw.get())
    }

    pub fn into_message(self) -> Result<WsMessageRef<'a>, KalshiError> {
        let WsEnvelopeRef {
            id,
            msg_type,
            sid,
            seq,
            msg,
            subscriptions,
        } = self;

        match msg_type {
            WsMsgType::Subscribed => Ok(WsMessageRef::Subscribed { id, sid }),
            WsMsgType::Unsubscribed => Ok(WsMessageRef::Unsubscribed { id, sid }),
            WsMsgType::Ok => Ok(WsMessageRef::Ok { id }),
            WsMsgType::ListSubscriptions => {
                let subs = if msg.is_some() {
                    let parsed: WsListSubscriptionsRef<'a> = parse_borrowed_msg(msg)?;
                    parsed.subscriptions
                } else {
                    subscriptions.unwrap_or_default()
                };
                Ok(WsMessageRef::ListSubscriptions {
                    id,
                    subscriptions: subs,
                })
            }
            WsMsgType::Error => {
                let error = if msg.is_some() {
                    parse_borrowed_msg::<WsErrorRef<'a>>(msg)?
                } else {
                    WsErrorRef {
                        code: None,
                        message: None,
                    }
                };
                Ok(WsMessageRef::Error { id, error })
            }
            WsMsgType::Ticker => Ok(WsMessageRef::Data(WsDataMessageRef::Ticker {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::TickerV2 => Ok(WsMessageRef::Data(WsDataMessageRef::TickerV2 {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::Trade => Ok(WsMessageRef::Data(WsDataMessageRef::Trade {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::OrderbookSnapshot => {
                Ok(WsMessageRef::Data(WsDataMessageRef::OrderbookSnapshot {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::OrderbookDelta => Ok(WsMessageRef::Data(WsDataMessageRef::OrderbookDelta {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::Fill => Ok(WsMessageRef::Data(WsDataMessageRef::Fill {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::MarketPositions => {
                Ok(WsMessageRef::Data(WsDataMessageRef::MarketPositions {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::MarketLifecycleV2 => {
                Ok(WsMessageRef::Data(WsDataMessageRef::MarketLifecycleV2 {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::Multivariate | WsMsgType::MultivariateLookup => {
                Ok(WsMessageRef::Data(WsDataMessageRef::Multivariate {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::RfqCreated => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::RfqCreated(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::RfqDeleted => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::RfqDeleted(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::QuoteCreated => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::QuoteCreated(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::QuoteAccepted => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::QuoteAccepted(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::QuoteExecuted => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::QuoteExecuted(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::OrderGroupUpdates => {
                Ok(WsMessageRef::Data(WsDataMessageRef::OrderGroupUpdates {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::Communications => Ok(WsMessageRef::Unknown {
                msg_type: WsMsgType::Communications,
                raw: msg,
            }),
            other => Ok(WsMessageRef::Unknown {
                msg_type: other,
                raw: msg,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsEnvelopeRef<'a> {
    pub id: Option<u64>,
    #[serde(rename = "type")]
    pub msg_type: WsMsgType,
    pub sid: Option<u64>,
    pub seq: Option<u64>,
    #[serde(borrow)]
    pub msg: Option<&'a RawValue>,
    #[serde(default, borrow)]
    pub subscriptions: Option<Vec<WsSubscriptionInfoRef<'a>>>,
}

fn parse_borrowed_msg<'a, T: Deserialize<'a>>(
    msg: Option<&'a RawValue>,
) -> Result<T, serde_json::Error> {
    let raw = msg.ok_or_else(|| serde_json::Error::custom("missing msg"))?;
    serde_json::from_str(raw.get())
}

impl<'a> WsEnvelopeRef<'a> {
    pub fn msg_raw(&self) -> Option<&str> {
        self.msg.as_deref().map(|raw| raw.get())
    }

    pub fn into_message(self) -> Result<WsMessageRef<'a>, KalshiError> {
        let WsEnvelopeRef {
            id,
            msg_type,
            sid,
            seq,
            msg,
            subscriptions,
        } = self;

        match msg_type {
            WsMsgType::Subscribed => Ok(WsMessageRef::Subscribed { id, sid }),
            WsMsgType::Unsubscribed => Ok(WsMessageRef::Unsubscribed { id, sid }),
            WsMsgType::Ok => Ok(WsMessageRef::Ok { id }),
            WsMsgType::ListSubscriptions => {
                let subs = if msg.is_some() {
                    let parsed: WsListSubscriptionsRef<'a> = parse_borrowed_msg(msg)?;
                    parsed.subscriptions
                } else {
                    subscriptions.unwrap_or_default()
                };
                Ok(WsMessageRef::ListSubscriptions {
                    id,
                    subscriptions: subs,
                })
            }
            WsMsgType::Error => {
                let error = if msg.is_some() {
                    parse_borrowed_msg::<WsErrorRef<'a>>(msg)?
                } else {
                    WsErrorRef {
                        code: None,
                        message: None,
                    }
                };
                Ok(WsMessageRef::Error { id, error })
            }
            WsMsgType::Ticker => Ok(WsMessageRef::Data(WsDataMessageRef::Ticker {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::TickerV2 => Ok(WsMessageRef::Data(WsDataMessageRef::TickerV2 {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::Trade => Ok(WsMessageRef::Data(WsDataMessageRef::Trade {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::OrderbookSnapshot => {
                Ok(WsMessageRef::Data(WsDataMessageRef::OrderbookSnapshot {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::OrderbookDelta => Ok(WsMessageRef::Data(WsDataMessageRef::OrderbookDelta {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::Fill => Ok(WsMessageRef::Data(WsDataMessageRef::Fill {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::MarketPositions => {
                Ok(WsMessageRef::Data(WsDataMessageRef::MarketPositions {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::MarketLifecycleV2 => {
                Ok(WsMessageRef::Data(WsDataMessageRef::MarketLifecycleV2 {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::EventLifecycle => Ok(WsMessageRef::Data(WsDataMessageRef::EventLifecycle {
                sid,
                seq,
                msg: parse_borrowed_msg(msg)?,
            })),
            WsMsgType::Multivariate | WsMsgType::MultivariateLookup => {
                Ok(WsMessageRef::Data(WsDataMessageRef::Multivariate {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::RfqCreated => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::RfqCreated(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::RfqDeleted => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::RfqDeleted(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::QuoteCreated => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::QuoteCreated(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::QuoteAccepted => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::QuoteAccepted(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::QuoteExecuted => Ok(WsMessageRef::Data(WsDataMessageRef::Communications {
                sid,
                seq,
                msg: WsCommunicationsRef::QuoteExecuted(parse_borrowed_msg(msg)?),
            })),
            WsMsgType::OrderGroupUpdates => {
                Ok(WsMessageRef::Data(WsDataMessageRef::OrderGroupUpdates {
                    sid,
                    seq,
                    msg: parse_borrowed_msg(msg)?,
                }))
            }
            WsMsgType::Communications => Ok(WsMessageRef::Unknown {
                msg_type: WsMsgType::Communications,
                raw: msg,
            }),
            other => Ok(WsMessageRef::Unknown {
                msg_type: other,
                raw: msg,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsListSubscriptions {
    #[serde(default)]
    pub subscriptions: Vec<WsSubscriptionInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsError {
    #[serde(default)]
    pub code: Option<i64>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum WsMessage {
    Subscribed {
        id: Option<u64>,
        sid: Option<u64>,
    },
    Unsubscribed {
        id: Option<u64>,
        sid: Option<u64>,
    },
    ListSubscriptions {
        id: Option<u64>,
        subscriptions: Vec<WsSubscriptionInfo>,
    },
    Ok {
        id: Option<u64>,
    },
    Error {
        id: Option<u64>,
        error: WsError,
    },
    Data(WsDataMessage),
    Unknown {
        msg_type: WsMsgType,
        raw: Option<Box<RawValue>>,
    },
}

#[derive(Debug, Clone)]
pub enum WsDataMessage {
    Ticker {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTicker,
    },
    TickerV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTickerV2,
    },
    Trade {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTrade,
    },
    OrderbookSnapshot {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookSnapshot,
    },
    OrderbookDelta {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookDelta,
    },
    Fill {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsFill,
    },
    MarketPositions {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketPositions,
    },
    MarketLifecycleV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketLifecycleV2,
    },
    EventLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsEventLifecycleV2,
    },
    Multivariate {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMultivariate,
    },
    Communications {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsCommunications,
    },
    OrderGroupUpdates {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderGroupUpdate,
    },
}

#[derive(Debug, Clone)]
pub enum WsDataMessageRef<'a> {
    Ticker {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTickerRef<'a>,
    },
    TickerV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTickerV2Ref<'a>,
    },
    Trade {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTradeRef<'a>,
    },
    OrderbookSnapshot {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookSnapshotRef<'a>,
    },
    OrderbookDelta {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookDeltaRef<'a>,
    },
    Fill {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsFillRef<'a>,
    },
    MarketPositions {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketPositionsRef<'a>,
    },
    MarketLifecycleV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketLifecycleV2Ref<'a>,
    },
    EventLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsEventLifecycleV2Ref<'a>,
    },
    Multivariate {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMultivariateRef<'a>,
    },
    Communications {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsCommunicationsRef<'a>,
    },
    OrderGroupUpdates {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderGroupUpdateRef<'a>,
    },
}

impl<'a> WsDataMessageRef<'a> {
    pub fn into_owned(self) -> WsDataMessage {
        match self {
            WsDataMessageRef::Ticker { sid, seq, msg } => WsDataMessage::Ticker {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::TickerV2 { sid, seq, msg } => WsDataMessage::TickerV2 {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::Trade { sid, seq, msg } => WsDataMessage::Trade {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::OrderbookSnapshot { sid, seq, msg } => {
                WsDataMessage::OrderbookSnapshot {
                    sid,
                    seq,
                    msg: msg.into_owned(),
                }
            }
            WsDataMessageRef::OrderbookDelta { sid, seq, msg } => WsDataMessage::OrderbookDelta {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::Fill { sid, seq, msg } => WsDataMessage::Fill {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::MarketPositions { sid, seq, msg } => WsDataMessage::MarketPositions {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::MarketLifecycleV2 { sid, seq, msg } => {
                WsDataMessage::MarketLifecycleV2 {
                    sid,
                    seq,
                    msg: msg.into_owned(),
                }
            }
            WsDataMessageRef::EventLifecycle { sid, seq, msg } => WsDataMessage::EventLifecycle {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::Multivariate { sid, seq, msg } => WsDataMessage::Multivariate {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::Communications { sid, seq, msg } => WsDataMessage::Communications {
                sid,
                seq,
                msg: msg.into_owned(),
            },
            WsDataMessageRef::OrderGroupUpdates { sid, seq, msg } => {
                WsDataMessage::OrderGroupUpdates {
                    sid,
                    seq,
                    msg: msg.into_owned(),
                }
            }
        }
    }
}

/// Borrowed WS message view.
///
/// Note: A smaller, purpose-built struct would be faster, but the library
/// prioritizes feature completeness across all message types.
#[derive(Debug, Clone)]
pub enum WsMessageRef<'a> {
    Subscribed {
        id: Option<u64>,
        sid: Option<u64>,
    },
    Unsubscribed {
        id: Option<u64>,
        sid: Option<u64>,
    },
    ListSubscriptions {
        id: Option<u64>,
        subscriptions: Vec<WsSubscriptionInfoRef<'a>>,
    },
    Ok {
        id: Option<u64>,
    },
    Error {
        id: Option<u64>,
        error: WsErrorRef<'a>,
    },
    Data(WsDataMessageRef<'a>),
    Unknown {
        msg_type: WsMsgType,
        raw: Option<&'a RawValue>,
    },
}

impl<'a> WsMessageRef<'a> {
    pub fn into_owned(self) -> Result<WsMessage, KalshiError> {
        let owned = match self {
            WsMessageRef::Subscribed { id, sid } => WsMessage::Subscribed { id, sid },
            WsMessageRef::Unsubscribed { id, sid } => WsMessage::Unsubscribed { id, sid },
            WsMessageRef::ListSubscriptions { id, subscriptions } => WsMessage::ListSubscriptions {
                id,
                subscriptions: subscriptions
                    .into_iter()
                    .map(WsSubscriptionInfoRef::into_owned)
                    .collect(),
            },
            WsMessageRef::Ok { id } => WsMessage::Ok { id },
            WsMessageRef::Error { id, error } => WsMessage::Error {
                id,
                error: error.into_owned(),
            },
            WsMessageRef::Data(data) => WsMessage::Data(data.into_owned()),
            WsMessageRef::Unknown { msg_type, raw } => {
                let raw_owned = match raw {
                    Some(value) => Some(serde_json::from_str::<Box<RawValue>>(value.get())?),
                    None => None,
                };
                WsMessage::Unknown {
                    msg_type,
                    raw: raw_owned,
                }
            }
        };
        Ok(owned)
    }
}

#[derive(Debug, Clone)]
pub struct WsRawEvent {
    bytes: Bytes,
}

impl WsRawEvent {
    pub fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }

    pub fn parse_owned(&self) -> Result<WsMessage, KalshiError> {
        WsMessage::from_bytes(&self.bytes)
    }

    pub fn parse_borrowed(&self) -> Result<WsMessageRef<'_>, KalshiError> {
        WsMessageRef::from_bytes(&self.bytes)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsSubscribeCmd {
    pub id: u64,
    pub cmd: &'static str, // "subscribe"
    pub params: WsSubscriptionParams,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsUnsubscribeCmd {
    pub id: u64,
    pub cmd: &'static str,
    pub params: WsUnsubscribeParams,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsUnsubscribeParams {
    pub sid: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsListSubscriptionsCmd {
    pub id: u64,
    pub cmd: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WsUpdateSubscriptionCmd {
    pub id: u64,
    pub cmd: &'static str,
    pub params: WsUpdateSubscriptionParams,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsUpdateSubscriptionParams {
    pub sid: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_tickers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_initial_snapshot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_factor: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_key: Option<String>,
}

pub(crate) fn validate_subscription(params: &WsSubscriptionParams) -> Result<(), KalshiError> {
    if params.channels.is_empty() {
        return Err(KalshiError::InvalidParams(
            "subscribe: at least one channel is required".to_string(),
        ));
    }

    let has_orderbook_delta = params
        .channels
        .iter()
        .any(|c| matches!(c, WsChannel::OrderbookDelta));
    let has_market_positions = params
        .channels
        .iter()
        .any(|c| matches!(c, WsChannel::MarketPositions));
    let has_communications = params
        .channels
        .iter()
        .any(|c| matches!(c, WsChannel::Communications));

    if has_orderbook_delta {
        let has_market_tickers = params
            .market_tickers
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let has_market_ids = params
            .market_ids
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if !(has_market_tickers || has_market_ids) {
            return Err(KalshiError::InvalidParams(
                "subscribe: orderbook_delta requires market_tickers or market_ids".to_string(),
            ));
        }
    }

    if params.send_initial_snapshot.is_some() && !has_orderbook_delta {
        return Err(KalshiError::InvalidParams(
            "subscribe: send_initial_snapshot only allowed for orderbook_delta".to_string(),
        ));
    }

    if params.market_ids.is_some() && has_market_positions {
        return Err(KalshiError::InvalidParams(
            "subscribe: market_positions only supports market_tickers".to_string(),
        ));
    }

    if (params.shard_factor.is_some() || params.shard_key.is_some()) && !has_communications {
        return Err(KalshiError::InvalidParams(
            "subscribe: shard_factor/shard_key only allowed for communications".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsWireMessage {
    #[serde(rename = "subscribed")]
    Subscribed { id: Option<u64>, sid: Option<u64> },
    #[serde(rename = "unsubscribed")]
    Unsubscribed { id: Option<u64>, sid: Option<u64> },
    #[serde(rename = "ok")]
    Ok { id: Option<u64> },
    #[serde(rename = "list_subscriptions")]
    ListSubscriptions {
        id: Option<u64>,
        #[serde(default)]
        subscriptions: Vec<WsSubscriptionInfo>,
        #[serde(default)]
        msg: Option<WsListSubscriptions>,
    },
    #[serde(rename = "error")]
    Error {
        id: Option<u64>,
        #[serde(default)]
        msg: Option<WsError>,
    },
    #[serde(rename = "ticker")]
    Ticker {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTicker,
    },
    #[serde(rename = "ticker_v2")]
    TickerV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTickerV2,
    },
    #[serde(rename = "trade")]
    Trade {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTrade,
    },
    #[serde(rename = "orderbook_snapshot")]
    OrderbookSnapshot {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookSnapshot,
    },
    #[serde(rename = "orderbook_delta")]
    OrderbookDelta {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookDelta,
    },
    #[serde(rename = "fill")]
    Fill {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsFill,
    },
    #[serde(rename = "market_positions")]
    MarketPositions {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketPositions,
    },
    #[serde(rename = "market_lifecycle_v2")]
    MarketLifecycleV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketLifecycleV2,
    },
    #[serde(rename = "event_lifecycle", alias = "event_lifecycle_v2")]
    EventLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsEventLifecycleV2,
    },
    #[serde(rename = "multivariate")]
    Multivariate {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMultivariate,
    },
    #[serde(rename = "multivariate_lookup")]
    MultivariateLookup {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMultivariate,
    },
    #[serde(rename = "rfq_created")]
    RfqCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsRfqCreated,
    },
    #[serde(rename = "rfq_deleted")]
    RfqDeleted {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsRfqDeleted,
    },
    #[serde(rename = "quote_created")]
    QuoteCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsQuoteCreated,
    },
    #[serde(rename = "quote_accepted")]
    QuoteAccepted {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsQuoteAccepted,
    },
    #[serde(rename = "quote_executed")]
    QuoteExecuted {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsQuoteExecuted,
    },
    #[serde(rename = "order_group_updates")]
    OrderGroupUpdates {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderGroupUpdate,
    },
}

impl WsWireMessage {
    fn into_message(self) -> WsMessage {
        match self {
            WsWireMessage::Subscribed { id, sid } => WsMessage::Subscribed { id, sid },
            WsWireMessage::Unsubscribed { id, sid } => WsMessage::Unsubscribed { id, sid },
            WsWireMessage::Ok { id } => WsMessage::Ok { id },
            WsWireMessage::ListSubscriptions {
                id,
                subscriptions,
                msg,
            } => {
                let subs = msg
                    .map(|value| value.subscriptions)
                    .unwrap_or(subscriptions);
                WsMessage::ListSubscriptions {
                    id,
                    subscriptions: subs,
                }
            }
            WsWireMessage::Error { id, msg } => WsMessage::Error {
                id,
                error: msg.unwrap_or(WsError {
                    code: None,
                    message: None,
                }),
            },
            WsWireMessage::Ticker { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Ticker { sid, seq, msg })
            }
            WsWireMessage::TickerV2 { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::TickerV2 { sid, seq, msg })
            }
            WsWireMessage::Trade { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Trade { sid, seq, msg })
            }
            WsWireMessage::OrderbookSnapshot { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::OrderbookSnapshot { sid, seq, msg })
            }
            WsWireMessage::OrderbookDelta { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::OrderbookDelta { sid, seq, msg })
            }
            WsWireMessage::Fill { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Fill { sid, seq, msg })
            }
            WsWireMessage::MarketPositions { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::MarketPositions { sid, seq, msg })
            }
            WsWireMessage::MarketLifecycleV2 { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::MarketLifecycleV2 { sid, seq, msg })
            }
            WsWireMessage::EventLifecycle { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::EventLifecycle { sid, seq, msg })
            }
            WsWireMessage::Multivariate { sid, seq, msg }
            | WsWireMessage::MultivariateLookup { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Multivariate { sid, seq, msg })
            }
            WsWireMessage::RfqCreated { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::RfqCreated(msg),
                })
            }
            WsWireMessage::RfqDeleted { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::RfqDeleted(msg),
                })
            }
            WsWireMessage::QuoteCreated { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::QuoteCreated(msg),
                })
            }
            WsWireMessage::QuoteAccepted { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::QuoteAccepted(msg),
                })
            }
            WsWireMessage::QuoteExecuted { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::QuoteExecuted(msg),
                })
            }
            WsWireMessage::OrderGroupUpdates { sid, seq, msg } => {
                WsMessage::Data(WsDataMessage::OrderGroupUpdates { sid, seq, msg })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsWireMessageRef<'a> {
    #[serde(rename = "subscribed")]
    Subscribed { id: Option<u64>, sid: Option<u64> },
    #[serde(rename = "unsubscribed")]
    Unsubscribed { id: Option<u64>, sid: Option<u64> },
    #[serde(rename = "ok")]
    Ok { id: Option<u64> },
    #[serde(rename = "list_subscriptions")]
    ListSubscriptions {
        id: Option<u64>,
        #[serde(default, borrow)]
        subscriptions: Vec<WsSubscriptionInfoRef<'a>>,
        #[serde(default, borrow)]
        msg: Option<WsListSubscriptionsRef<'a>>,
    },
    #[serde(rename = "error")]
    Error {
        id: Option<u64>,
        #[serde(default, borrow)]
        msg: Option<WsErrorRef<'a>>,
    },
    #[serde(rename = "ticker")]
    Ticker {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsTickerRef<'a>,
    },
    #[serde(rename = "ticker_v2")]
    TickerV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsTickerV2Ref<'a>,
    },
    #[serde(rename = "trade")]
    Trade {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsTradeRef<'a>,
    },
    #[serde(rename = "orderbook_snapshot")]
    OrderbookSnapshot {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsOrderbookSnapshotRef<'a>,
    },
    #[serde(rename = "orderbook_delta")]
    OrderbookDelta {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsOrderbookDeltaRef<'a>,
    },
    #[serde(rename = "fill")]
    Fill {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsFillRef<'a>,
    },
    #[serde(rename = "market_positions")]
    MarketPositions {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMarketPositionsRef<'a>,
    },
    #[serde(rename = "market_lifecycle_v2")]
    MarketLifecycleV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMarketLifecycleV2Ref<'a>,
    },
    #[serde(rename = "event_lifecycle", alias = "event_lifecycle_v2")]
    EventLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsEventLifecycleV2Ref<'a>,
    },
    #[serde(rename = "multivariate")]
    Multivariate {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMultivariateRef<'a>,
    },
    #[serde(rename = "multivariate_lookup")]
    MultivariateLookup {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMultivariateRef<'a>,
    },
    #[serde(rename = "rfq_created")]
    RfqCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsRfqCreatedRef<'a>,
    },
    #[serde(rename = "rfq_deleted")]
    RfqDeleted {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsRfqDeletedRef<'a>,
    },
    #[serde(rename = "quote_created")]
    QuoteCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsQuoteCreatedRef<'a>,
    },
    #[serde(rename = "quote_accepted")]
    QuoteAccepted {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsQuoteAcceptedRef<'a>,
    },
    #[serde(rename = "quote_executed")]
    QuoteExecuted {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsQuoteExecutedRef<'a>,
    },
    #[serde(rename = "order_group_updates")]
    OrderGroupUpdates {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsOrderGroupUpdateRef<'a>,
    },
}

impl<'a> WsWireMessageRef<'a> {
    fn into_message(self) -> WsMessageRef<'a> {
        match self {
            WsWireMessageRef::Subscribed { id, sid } => WsMessageRef::Subscribed { id, sid },
            WsWireMessageRef::Unsubscribed { id, sid } => WsMessageRef::Unsubscribed { id, sid },
            WsWireMessageRef::Ok { id } => WsMessageRef::Ok { id },
            WsWireMessageRef::ListSubscriptions {
                id,
                subscriptions,
                msg,
            } => {
                let subs = msg
                    .map(|value| value.subscriptions)
                    .unwrap_or(subscriptions);
                WsMessageRef::ListSubscriptions {
                    id,
                    subscriptions: subs,
                }
            }
            WsWireMessageRef::Error { id, msg } => WsMessageRef::Error {
                id,
                error: msg.unwrap_or(WsErrorRef {
                    code: None,
                    message: None,
                }),
            },
            WsWireMessageRef::Ticker { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Ticker { sid, seq, msg })
            }
            WsWireMessageRef::TickerV2 { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::TickerV2 { sid, seq, msg })
            }
            WsWireMessageRef::Trade { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Trade { sid, seq, msg })
            }
            WsWireMessageRef::OrderbookSnapshot { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::OrderbookSnapshot { sid, seq, msg })
            }
            WsWireMessageRef::OrderbookDelta { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::OrderbookDelta { sid, seq, msg })
            }
            WsWireMessageRef::Fill { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Fill { sid, seq, msg })
            }
            WsWireMessageRef::MarketPositions { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::MarketPositions { sid, seq, msg })
            }
            WsWireMessageRef::MarketLifecycleV2 { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::MarketLifecycleV2 { sid, seq, msg })
            }
            WsWireMessageRef::EventLifecycle { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::EventLifecycle { sid, seq, msg })
            }
            WsWireMessageRef::Multivariate { sid, seq, msg }
            | WsWireMessageRef::MultivariateLookup { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Multivariate { sid, seq, msg })
            }
            WsWireMessageRef::RfqCreated { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::RfqCreated(msg),
                })
            }
            WsWireMessageRef::RfqDeleted { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::RfqDeleted(msg),
                })
            }
            WsWireMessageRef::QuoteCreated { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::QuoteCreated(msg),
                })
            }
            WsWireMessageRef::QuoteAccepted { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::QuoteAccepted(msg),
                })
            }
            WsWireMessageRef::QuoteExecuted { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::QuoteExecuted(msg),
                })
            }
            WsWireMessageRef::OrderGroupUpdates { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::OrderGroupUpdates { sid, seq, msg })
            }
        }
    }
}

impl WsMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KalshiError> {
        match serde_json::from_slice::<WsWireMessage>(bytes) {
            Ok(wire) => Ok(wire.into_message()),
            Err(first_err) => match serde_json::from_slice::<WsEnvelope>(bytes) {
                Ok(env) => env.into_message(),
                Err(_) => Err(first_err.into()),
            },
        }
    }
}

impl<'a> WsMessageRef<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, KalshiError> {
        match serde_json::from_slice::<WsWireMessageRef<'a>>(bytes) {
            Ok(wire) => Ok(wire.into_message()),
            Err(first_err) => match serde_json::from_slice::<WsEnvelopeRef<'a>>(bytes) {
                Ok(env) => env.into_message(),
                Err(_) => Err(first_err.into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn validate_subscription_requires_market_tickers_for_orderbook_delta() {
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::OrderbookDelta],
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_err());

        let params = WsSubscriptionParams {
            channels: vec![WsChannel::OrderbookDelta],
            market_tickers: Some(vec!["TEST".to_string()]),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_ok());
    }

    #[test]
    fn validate_subscription_send_initial_snapshot_only_for_orderbook_delta() {
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::Ticker],
            send_initial_snapshot: Some(true),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_err());
    }

    #[test]
    fn validate_subscription_orderbook_delta_allows_market_ids() {
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::OrderbookDelta],
            market_ids: Some(vec!["mid-1".to_string()]),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_ok());
    }

    #[test]
    fn validate_subscription_rejects_market_positions_with_market_ids() {
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::MarketPositions],
            market_ids: Some(vec!["mid-1".to_string()]),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_err());
    }

    #[test]
    fn validate_subscription_shard_fields_require_communications() {
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::Ticker],
            shard_factor: Some(2),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_err());

        let params = WsSubscriptionParams {
            channels: vec![WsChannel::Communications],
            shard_factor: Some(2),
            shard_key: Some("key".to_string()),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_ok());
    }

    #[test]
    fn validate_subscription_send_initial_snapshot_with_orderbook_delta_ok() {
        let params = WsSubscriptionParams {
            channels: vec![WsChannel::OrderbookDelta],
            market_tickers: Some(vec!["TEST".to_string()]),
            send_initial_snapshot: Some(true),
            ..Default::default()
        };
        assert!(validate_subscription(&params).is_ok());
    }

    #[test]
    fn ws_msg_type_deserialize_known() {
        let msg_type: WsMsgType = serde_json::from_str("\"trade\"").unwrap();
        assert!(matches!(msg_type, WsMsgType::Trade));
    }

    #[test]
    fn ws_msg_type_deserialize_unknown() {
        let msg_type: WsMsgType = serde_json::from_str("\"new_type\"").unwrap();
        assert!(matches!(msg_type, WsMsgType::Unknown(value) if value == "new_type"));
    }

    #[test]
    fn ws_envelope_into_message_known_type() {
        let json = r#"{
            "type":"ticker",
            "sid":1,
            "seq":2,
            "msg":{
                "market_ticker":"TEST",
                "market_id":"1",
                "price":1,
                "yes_bid":1,
                "yes_ask":2,
                "price_dollars":"0.01",
                "yes_bid_dollars":"0.01",
                "yes_ask_dollars":"0.02",
                "volume":0,
                "volume_fp":"0",
                "open_interest":0,
                "open_interest_fp":"0",
                "dollar_volume":0,
                "dollar_open_interest":0,
                "ts":0
            }
        }"#;
        let env: WsEnvelope = serde_json::from_str(json).unwrap();
        let msg = env.into_message().unwrap();
        assert!(matches!(msg, WsMessage::Data(WsDataMessage::Ticker { .. })));
    }

    #[test]
    fn ws_envelope_into_message_unknown_type() {
        let json = r#"{"type":"mystery","msg":{"foo":1}}"#;
        let env: WsEnvelope = serde_json::from_str(json).unwrap();
        let msg = env.into_message().unwrap();
        match msg {
            WsMessage::Unknown {
                msg_type: WsMsgType::Unknown(value),
                raw,
            } => {
                assert_eq!(value, "mystery");
                assert!(raw.is_some());
            }
            _ => panic!("expected unknown message"),
        }
    }

    #[test]
    fn ws_message_from_bytes_known_type() {
        let json = r#"{
            "type":"ticker",
            "sid":1,
            "seq":2,
            "msg":{
                "market_ticker":"TEST",
                "market_id":"1",
                "price":1,
                "yes_bid":1,
                "yes_ask":2,
                "price_dollars":"0.01",
                "yes_bid_dollars":"0.01",
                "yes_ask_dollars":"0.02",
                "volume":0,
                "volume_fp":"0",
                "open_interest":0,
                "open_interest_fp":"0",
                "dollar_volume":0,
                "dollar_open_interest":0,
                "ts":0
            }
        }"#;
        let msg = WsMessage::from_bytes(json.as_bytes()).unwrap();
        assert!(matches!(msg, WsMessage::Data(WsDataMessage::Ticker { .. })));
    }

    #[test]
    fn ws_message_from_bytes_unknown_type() {
        let json = r#"{"type":"mystery","msg":{"foo":1}}"#;
        let msg = WsMessage::from_bytes(json.as_bytes()).unwrap();
        match msg {
            WsMessage::Unknown {
                msg_type: WsMsgType::Unknown(value),
                raw,
            } => {
                assert_eq!(value, "mystery");
                assert!(raw.is_some());
            }
            _ => panic!("expected unknown message"),
        }
    }

    #[test]
    fn ws_message_ref_roundtrip_owned() {
        let json = r#"{
            "type":"trade",
            "sid":3,
            "seq":4,
            "msg":{
                "trade_id":"t1",
                "ticker":"TST",
                "price":10,
                "count":2,
                "count_fp":"2",
                "yes_price":10,
                "no_price":90,
                "yes_price_dollars":"0.10",
                "no_price_dollars":"0.90",
                "taker_side":"yes",
                "created_time":"2024-01-01T00:00:00Z"
            }
        }"#;
        let msg_ref = WsMessageRef::from_bytes(json.as_bytes()).unwrap();
        let msg = msg_ref.into_owned().unwrap();
        assert!(matches!(msg, WsMessage::Data(WsDataMessage::Trade { .. })));
    }

    #[test]
    fn ws_raw_event_parse_borrowed() {
        let json = r#"{
            "type":"ticker_v2",
            "sid":9,
            "seq":10,
            "msg":{
                "market_ticker":"TEST",
                "price":1,
                "price_dollars":"0.01",
                "ts":123
            }
        }"#;
        let raw = WsRawEvent::new(Bytes::from(json));
        let msg = raw.parse_borrowed().unwrap();
        assert!(matches!(
            msg,
            WsMessageRef::Data(WsDataMessageRef::TickerV2 { .. })
        ));
    }

    #[test]
    fn ws_orderbook_delta_side_parse() {
        let json = r#"{
            "market_ticker":"TEST",
            "market_id":"1",
            "price":1,
            "price_dollars":"0.01",
            "delta":1,
            "delta_fp":"1",
            "side":"yes"
        }"#;
        let delta: WsOrderbookDelta = serde_json::from_str(json).unwrap();
        assert!(matches!(delta.side, YesNo::Yes));
    }

    #[test]
    fn ws_fill_side_action_parse() {
        let json = r#"{
            "fill_id":"f",
            "trade_id":"t",
            "order_id":"o",
            "ticker":"T",
            "market_ticker":"M",
            "side":"no",
            "action":"buy",
            "count":1,
            "count_fp":"1",
            "yes_price":1,
            "no_price":2,
            "yes_price_dollars":"0.01",
            "no_price_dollars":"0.02",
            "is_taker":true,
            "fee_cost":"0.00"
        }"#;
        let fill: WsFill = serde_json::from_str(json).unwrap();
        assert!(matches!(fill.side, YesNo::No));
        assert!(matches!(fill.action, BuySell::Buy));
    }
}
