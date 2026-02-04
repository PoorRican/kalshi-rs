use crate::error::KalshiError;
use crate::rest::types::{EventPosition, MarketPosition};
use crate::types::{AnyJson, TradeTakerSide};

use serde::{Deserialize, Serialize};
use serde::de::Error as _;
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
    pub side: String,
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
    pub side: String,
    pub action: String,
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
    pub status: Option<String>,
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

/// Communications message payload (type: "communications")
pub type WsCommunications = AnyJson;

/// Multivariate message payload (type: "multivariate")
pub type WsMultivariate = AnyJson;

/// Order group update message payload (type: "order_group_updates")
pub type WsOrderGroupUpdate = AnyJson;

/// Envelope used by Kalshi WS (data + errors use "type")
#[derive(Debug, Clone, Deserialize)]
pub struct WsEnvelope {
    pub id: Option<u64>,
    #[serde(rename = "type")]
    pub msg_type: String,
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

    fn parse_msg<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        let raw = self
            .msg
            .as_deref()
            .ok_or_else(|| serde_json::Error::custom("missing msg"))?;
        serde_json::from_str(raw.get())
    }

    pub fn into_message(mut self) -> Result<WsMessage, KalshiError> {
        let msg_type = std::mem::take(&mut self.msg_type);
        match msg_type.as_str() {
            "subscribed" => Ok(WsMessage::Subscribed {
                id: self.id,
                sid: self.sid,
            }),
            "unsubscribed" => Ok(WsMessage::Unsubscribed {
                id: self.id,
                sid: self.sid,
            }),
            "ok" => Ok(WsMessage::Ok { id: self.id }),
            "list_subscriptions" => {
                let subs = if let Some(raw) = self.msg {
                    let parsed: WsListSubscriptions = serde_json::from_str(raw.get())?;
                    parsed.subscriptions
                } else {
                    self.subscriptions.unwrap_or_default()
                };
                Ok(WsMessage::ListSubscriptions {
                    id: self.id,
                    subscriptions: subs,
                })
            }
            "error" => {
                let error = if let Some(raw) = self.msg {
                    serde_json::from_str(raw.get())?
                } else {
                    WsError { code: None, message: None }
                };
                Ok(WsMessage::Error { id: self.id, error })
            }
            "ticker" => Ok(WsMessage::Data(WsDataMessage::Ticker {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "ticker_v2" => Ok(WsMessage::Data(WsDataMessage::TickerV2 {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "trade" => Ok(WsMessage::Data(WsDataMessage::Trade {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "orderbook_snapshot" => Ok(WsMessage::Data(WsDataMessage::OrderbookSnapshot {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "orderbook_delta" => Ok(WsMessage::Data(WsDataMessage::OrderbookDelta {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "fill" => Ok(WsMessage::Data(WsDataMessage::Fill {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "market_positions" => Ok(WsMessage::Data(WsDataMessage::MarketPositions {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "market_lifecycle_v2" => Ok(WsMessage::Data(WsDataMessage::MarketLifecycleV2 {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "multivariate" => Ok(WsMessage::Data(WsDataMessage::Multivariate {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "communications" => Ok(WsMessage::Data(WsDataMessage::Communications {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            "order_group_updates" => Ok(WsMessage::Data(WsDataMessage::OrderGroupUpdates {
                sid: self.sid,
                seq: self.seq,
                msg: self.parse_msg()?,
            })),
            _ => Ok(WsMessage::Unknown {
                msg_type,
                raw: self.msg,
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
    Subscribed { id: Option<u64>, sid: Option<u64> },
    Unsubscribed { id: Option<u64>, sid: Option<u64> },
    ListSubscriptions { id: Option<u64>, subscriptions: Vec<WsSubscriptionInfo> },
    Ok { id: Option<u64> },
    Error { id: Option<u64>, error: WsError },
    Data(WsDataMessage),
    Unknown { msg_type: String, raw: Option<Box<RawValue>> },
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
}
