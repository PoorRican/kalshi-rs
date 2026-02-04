use crate::auth::KalshiAuth;
use crate::env::{KalshiEnvironment, WS_PATH};
use crate::error::KalshiError;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::fmt;

use tokio_tungstenite::tungstenite::http::{HeaderValue, Request};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
>;

pub struct KalshiWsClient {
    write: futures::stream::SplitSink<WsStream, Message>,
    read: futures::stream::SplitStream<WsStream>,
    next_id: u64,
    authenticated: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WsChannel {
    // Public (no auth required) :contentReference[oaicite:71]{index=71}
    Ticker,
    TickerV2,
    Trade,
    MarketLifecycleV2,
    Multivariate,

    // Private (auth required) :contentReference[oaicite:72]{index=72}
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

impl Serialize for WsChannel {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
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
    pub yes_price_fixed: String,
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

/// Envelope used by Kalshi WS (data + errors use "type")
#[derive(Debug, Clone, Deserialize)]
pub struct WsEnvelope {
    pub id: Option<u64>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub sid: Option<u64>,
    pub seq: Option<u64>,
    pub msg: Option<serde_json::Value>,
}

impl WsEnvelope {
    /// Parse inner message as a ticker update.
    pub fn parse_ticker(&self) -> Result<WsTicker, serde_json::Error> {
        serde_json::from_value(self.msg.clone().unwrap_or_default())
    }

    /// Parse inner message as an orderbook snapshot.
    pub fn parse_orderbook_snapshot(&self) -> Result<WsOrderbookSnapshot, serde_json::Error> {
        serde_json::from_value(self.msg.clone().unwrap_or_default())
    }

    /// Parse inner message as an orderbook delta.
    pub fn parse_orderbook_delta(&self) -> Result<WsOrderbookDelta, serde_json::Error> {
        serde_json::from_value(self.msg.clone().unwrap_or_default())
    }

    /// Parse inner message as a fill.
    pub fn parse_fill(&self) -> Result<WsFill, serde_json::Error> {
        serde_json::from_value(self.msg.clone().unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize)]
struct WsSubscribeCmd {
    pub id: u64,
    pub cmd: &'static str, // "subscribe"
    pub params: WsSubscribeParams,
}

#[derive(Debug, Clone, Serialize)]
struct WsSubscribeParams {
    pub channels: Vec<WsChannel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
}

impl KalshiWsClient {
    /// Connect without auth (public channels only). :contentReference[oaicite:74]{index=74}
    pub async fn connect(env: KalshiEnvironment) -> Result<Self, KalshiError> {
        let (ws_stream, _resp) = connect_async(&env.ws_url)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            next_id: 1,
            authenticated: false,
        })
    }

    /// Connect with auth headers so you can subscribe to private channels. :contentReference[oaicite:75]{index=75}
    pub async fn connect_authenticated(env: KalshiEnvironment, auth: KalshiAuth) -> Result<Self, KalshiError> {
        let mut req: Request<()> = env
            .ws_url
            .into_client_request()
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        // WS signing: timestamp + "GET" + "/trade-api/ws/v2" :contentReference[oaicite:76]{index=76}
        let headers = auth.build_headers("GET", WS_PATH)?;

        req.headers_mut().insert(
            "KALSHI-ACCESS-KEY",
            HeaderValue::from_str(&headers.key).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-SIGNATURE",
            HeaderValue::from_str(&headers.signature).map_err(|e| KalshiError::Header(e.to_string()))?,
        );
        req.headers_mut().insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&headers.timestamp_ms).map_err(|e| KalshiError::Header(e.to_string()))?,
        );

        let (ws_stream, _resp) = connect_async(req)
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        let (write, read) = ws_stream.split();
        Ok(Self {
            write,
            read,
            next_id: 1,
            authenticated: true,
        })
    }

    /// Subscribe to channels; add `market_tickers` when required (e.g. orderbook_delta). :contentReference[oaicite:77]{index=77}
    pub async fn subscribe(
        &mut self,
        channels: Vec<WsChannel>,
        market_tickers: Option<Vec<String>>,
    ) -> Result<u64, KalshiError> {
        let needs_auth = channels.iter().any(|c| c.is_private());
        if needs_auth && !self.authenticated {
            // Server would emit code 9 "Authentication required" :contentReference[oaicite:78]{index=78}
            return Err(KalshiError::AuthRequired("WebSocket private channel subscription"));
        }

        let id = self.next_id;
        self.next_id += 1;

        let cmd = WsSubscribeCmd {
            id,
            cmd: "subscribe",
            params: WsSubscribeParams {
                channels,
                market_tickers,
            },
        };

        let text = serde_json::to_string(&cmd)?;
        self.write
            .send(Message::Text(text))
            .await
            .map_err(|e| KalshiError::Ws(e.to_string()))?;

        Ok(id)
    }

    /// Read the next JSON message from the stream.
    pub async fn next_envelope(&mut self) -> Result<WsEnvelope, KalshiError> {
        while let Some(msg) = self.read.next().await {
            let msg = msg.map_err(|e| KalshiError::Ws(e.to_string()))?;
            match msg {
                Message::Text(s) => return Ok(serde_json::from_str::<WsEnvelope>(&s)?),
                Message::Binary(b) => {
                    // If server ever sends binary JSON, attempt decode.
                    let s = String::from_utf8(b).map_err(|e| KalshiError::Ws(e.to_string()))?;
                    return Ok(serde_json::from_str::<WsEnvelope>(&s)?);
                }
                Message::Ping(payload) => {
                    self.write
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|e| KalshiError::Ws(e.to_string()))?;
                    self.write
                        .flush()
                        .await
                        .map_err(|e| KalshiError::Ws(e.to_string()))?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => return Err(KalshiError::Ws("websocket closed".to_string())),
                _ => {}
            }
        }
        Err(KalshiError::Ws("websocket stream ended".to_string()))
    }
}

