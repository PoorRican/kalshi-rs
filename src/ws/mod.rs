//! WebSocket client for the Kalshi real-time streaming API.
//!
//! Two client tiers are provided:
//!
//! | Client | Reconnect | Resubscribe | Use case |
//! |--------|-----------|-------------|----------|
//! | [`KalshiWsClient`] | Automatic | Automatic | Most applications |
//! | [`KalshiWsLowLevelClient`] | Manual | Manual | Custom reconnect logic |
//!
//! # Channels
//!
//! | Channel | Auth | Description |
//! |---------|------|-------------|
//! | [`WsChannel::Ticker`] | No | Price / volume snapshots |
//! | [`WsChannel::TickerV2`] | No | Delta-style ticker updates |
//! | [`WsChannel::Trade`] | No | Public trades |
//! | [`WsChannel::MarketLifecycleV2`] | No | Market open / close / settle events |
//! | [`WsChannel::Multivariate`] | No | Multivariate market lookups |
//! | [`WsChannel::OrderbookDelta`] | Yes | L2 order-book deltas (requires `market_tickers`) |
//! | [`WsChannel::Fill`] | Yes | Your fills |
//! | [`WsChannel::MarketPositions`] | Yes | Position changes |
//! | [`WsChannel::Communications`] | Yes | RFQs and quotes |
//! | [`WsChannel::OrderGroupUpdates`] | Yes | Order-group lifecycle |
//!
//! # Quick Start — Public Ticker
//!
//! ```no_run
//! use kalshi_fast::{
//!     KalshiEnvironment, KalshiWsClient, WsChannel, WsDataMessage,
//!     WsEvent, WsMessage, WsReconnectConfig, WsSubscriptionParams,
//! };
//!
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! let mut ws = KalshiWsClient::connect(
//!     KalshiEnvironment::demo(),
//!     WsReconnectConfig::default(),
//! ).await?;
//!
//! ws.subscribe(WsSubscriptionParams {
//!     channels: vec![WsChannel::Ticker],
//!     ..Default::default()
//! }).await?;
//!
//! loop {
//!     match ws.next_event().await? {
//!         WsEvent::Message(WsMessage::Data(WsDataMessage::Ticker { msg, .. })) => {
//!             println!("{}: {}", msg.market_ticker, msg.price);
//!         }
//!         WsEvent::Reconnected { attempt } => println!("Reconnected (attempt {attempt})"),
//!         WsEvent::Disconnected { .. } => break,
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Authenticated — Order-Book Deltas
//!
//! Private channels require [`KalshiWsClient::connect_authenticated`]:
//!
//! ```no_run
//! use kalshi_fast::{
//!     KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannel,
//!     WsDataMessage, WsEvent, WsMessage, WsReconnectConfig, WsSubscriptionParams,
//! };
//!
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! let auth = KalshiAuth::from_pem_file(
//!     std::env::var("KALSHI_KEY_ID").unwrap(),
//!     std::env::var("KALSHI_PRIVATE_KEY_PATH").unwrap(),
//! )?;
//!
//! let mut ws = KalshiWsClient::connect_authenticated(
//!     KalshiEnvironment::demo(),
//!     auth,
//!     WsReconnectConfig::default(),
//! ).await?;
//!
//! ws.subscribe(WsSubscriptionParams {
//!     channels: vec![WsChannel::OrderbookDelta],
//!     market_tickers: Some(vec!["SOME-MARKET".into()]),
//!     ..Default::default()
//! }).await?;
//!
//! loop {
//!     match ws.next_event().await? {
//!         WsEvent::Message(WsMessage::Data(WsDataMessage::OrderbookDelta { msg, .. })) => {
//!             println!("{} {} delta={}", msg.market_ticker, msg.side, msg.delta);
//!         }
//!         WsEvent::Disconnected { .. } => break,
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Message Flow
//!
//! ```text
//! next_event() → WsEvent
//!                 ├─ Message(WsMessage)
//!                 │    ├─ Data(WsDataMessage::Ticker { .. })
//!                 │    ├─ Data(WsDataMessage::Fill { .. })
//!                 │    ├─ Subscribed / Unsubscribed / Ok
//!                 │    ├─ Error { .. }
//!                 │    └─ Unknown { .. }
//!                 ├─ Reconnected { attempt }
//!                 └─ Disconnected { error }
//! ```
//!
//! # Reconnection
//!
//! [`KalshiWsClient`] reconnects automatically with exponential backoff when
//! the underlying connection drops. On success it resubscribes to all active
//! channels and emits [`WsEvent::Reconnected`]. If retries are exhausted it
//! emits [`WsEvent::Disconnected`]. Configure via [`WsReconnectConfig`].
//!
//! **Note:** Sequence resync is not automatic; callers must handle any gaps
//! using the `seq` field on [`WsDataMessage`] variants.

mod client;
pub mod types;

pub use client::{
    KalshiWsClient, KalshiWsLowLevelClient, WsEvent, WsEventReceiver, WsReaderConfig, WsReaderMode,
    WsReconnectConfig,
};
pub use types::*;
