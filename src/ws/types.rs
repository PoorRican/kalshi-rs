use crate::error::KalshiError;
use crate::rest::types::{EventPosition, MarketPosition};
use crate::types::{
    BuySell, FixedPointCount, FixedPointDollars, MarketStatus, TradeTakerSide, YesNo,
};

use serde::{Deserialize, Serialize};
use serde::de::{Error as _, Visitor};
use serde_json::value::RawValue;
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

            fn visit_borrowed_str<E: serde::de::Error>(self, value: &'de str) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_str(value).unwrap_or_else(|| WsMsgType::Unknown(value.to_owned())))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_str(value).unwrap_or_else(|| WsMsgType::Unknown(value.to_owned())))
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
    /// market status
    #[serde(default)]
    pub status: Option<MarketStatus>,
    #[serde(default)]
    pub can_trade: Option<bool>,
    #[serde(default)]
    pub can_settle: Option<bool>,
    #[serde(default)]
    pub open_time: Option<String>,
    #[serde(default)]
    pub close_time: Option<String>,
    #[serde(default)]
    pub settled_time: Option<String>,
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
                Ok(WsMessage::ListSubscriptions { id, subscriptions: subs })
            }
            WsMsgType::Error => {
                let error = if msg.is_some() {
                    parse_msg(&msg)?
                } else {
                    WsError { code: None, message: None }
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
            other => Ok(WsMessage::Unknown { msg_type: other, raw: msg }),
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
    Subscribed { id: Option<u64>, sid: Option<u64> },
    Unsubscribed { id: Option<u64>, sid: Option<u64> },
    ListSubscriptions { id: Option<u64>, subscriptions: Vec<WsSubscriptionInfo> },
    Ok { id: Option<u64> },
    Error { id: Option<u64>, error: WsError },
    Data(WsDataMessage),
    Unknown { msg_type: WsMsgType, raw: Option<Box<RawValue>> },
}

#[derive(Debug, Clone)]
pub enum WsDataMessage {
    Ticker { sid: Option<u64>, seq: Option<u64>, msg: WsTicker },
    TickerV2 { sid: Option<u64>, seq: Option<u64>, msg: WsTickerV2 },
    Trade { sid: Option<u64>, seq: Option<u64>, msg: WsTrade },
    OrderbookSnapshot { sid: Option<u64>, seq: Option<u64>, msg: WsOrderbookSnapshot },
    OrderbookDelta { sid: Option<u64>, seq: Option<u64>, msg: WsOrderbookDelta },
    Fill { sid: Option<u64>, seq: Option<u64>, msg: WsFill },
    MarketPositions { sid: Option<u64>, seq: Option<u64>, msg: WsMarketPositions },
    MarketLifecycleV2 { sid: Option<u64>, seq: Option<u64>, msg: WsMarketLifecycleV2 },
    Multivariate { sid: Option<u64>, seq: Option<u64>, msg: WsMultivariate },
    Communications { sid: Option<u64>, seq: Option<u64>, msg: WsCommunications },
    OrderGroupUpdates { sid: Option<u64>, seq: Option<u64>, msg: WsOrderGroupUpdate },
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

    let has_orderbook_delta = params.channels.iter().any(|c| matches!(c, WsChannel::OrderbookDelta));
    let has_market_positions = params.channels.iter().any(|c| matches!(c, WsChannel::MarketPositions));
    let has_communications = params.channels.iter().any(|c| matches!(c, WsChannel::Communications));

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

#[cfg(test)]
mod tests {
    use super::*;

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
            WsMessage::Unknown { msg_type: WsMsgType::Unknown(value), raw } => {
                assert_eq!(value, "mystery");
                assert!(raw.is_some());
            }
            _ => panic!("expected unknown message"),
        }
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
