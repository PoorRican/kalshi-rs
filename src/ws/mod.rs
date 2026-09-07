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
//! | [`WsChannelV2::Ticker`] | No | Price / volume snapshots |
//! | [`WsChannelV2::Trade`] | No | Public trades |
//! | [`WsChannelV2::MarketLifecycleV2`] | No | Market open / close / settle events |
//! | [`WsChannelV2::MultivariateMarketLifecycle`] | No | Multivariate market lifecycle events |
//! | [`WsChannelV2::OrderbookDelta`] | Yes | L2 order-book deltas (requires `market_tickers`) |
//! | [`WsChannelV2::Fill`] | Yes | Your fills |
//! | [`WsChannelV2::MarketPositions`] | Yes | Position changes |
//! | [`WsChannelV2::Communications`] | Yes | RFQs and quotes |
//! | [`WsChannelV2::OrderGroupUpdates`] | Yes | Order-group lifecycle |
//! | [`WsChannelV2::UserOrders`] | Yes | User order lifecycle updates |
//!
//! # Quick Start — Public Ticker
//!
//! ```no_run
//! use kalshi_fast::{
//!     KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannelV2, WsDataMessageV2,
//!     WsEvent, WsMessageV2, WsReconnectConfig, WsSubscriptionParamsV2,
//! };
//!
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! let auth = KalshiAuth::from_pem_file(
//!     std::env::var("KALSHI_KEY_ID").unwrap(),
//!     std::env::var("KALSHI_PRIVATE_KEY_PATH").unwrap(),
//! )?;
//! let mut ws = KalshiWsClient::connect_authenticated(
//!     KalshiEnvironment::demo(),
//!     auth,
//!     WsReconnectConfig::default(),
//! ).await?;
//!
//! ws.subscribe_v2(WsSubscriptionParamsV2 {
//!     channels: vec![WsChannelV2::Ticker],
//!     ..Default::default()
//! }).await?;
//!
//! loop {
//!     match ws.next_event_v2().await? {
//!         WsEvent::Message(WsMessageV2::Data(WsDataMessageV2::Ticker { msg, .. })) => {
//!             println!("{}: {}", msg.market_ticker, msg.price_dollars);
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
//! Kalshi requires [`KalshiWsClient::connect_authenticated`] for every WebSocket
//! connection, including public channels:
//!
//! ```no_run
//! use kalshi_fast::{
//!     KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannelV2,
//!     WsDataMessageV2, WsEvent, WsMessageV2, WsReconnectConfig, WsSubscriptionParamsV2,
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
//! ws.subscribe_v2(WsSubscriptionParamsV2 {
//!     channels: vec![WsChannelV2::OrderbookDelta],
//!     market_tickers: Some(vec!["SOME-MARKET".into()]),
//!     ..Default::default()
//! }).await?;
//!
//! loop {
//!     match ws.next_event_v2().await? {
//!         WsEvent::Message(WsMessageV2::Data(WsDataMessageV2::OrderbookDelta { msg, .. })) => {
//!             println!("{} {} delta={}", msg.market_ticker, msg.side, msg.delta_fp);
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
//! next_event_v2() → WsEvent
//!                 ├─ Message(WsMessageV2)
//!                 │    ├─ Data(WsDataMessageV2::Ticker { .. })
//!                 │    ├─ Data(WsDataMessageV2::Fill { .. })
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
//! **Note:** Sequence resync is not automatic; callers must detect gaps with
//! [`WsMessageV2::subscription_id`] and [`WsMessageV2::sequence`]. Matching only
//! [`WsDataMessageV2`] variants is unsafe because control and future unknown frames can also
//! consume the per-subscription cursor.

mod client;
mod event;
mod low_level;
pub(crate) mod reader;
mod reconnect;
pub(crate) mod subscription;
pub mod types;

pub use client::*;
pub use event::*;
pub use low_level::*;
pub use reconnect::*;
pub use types::*;
