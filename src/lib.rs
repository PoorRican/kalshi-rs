//! # kalshi-fast-rs
//!
//! High-performance async Rust client for the [Kalshi](https://kalshi.com) prediction markets API.
//!
#![allow(
    clippy::large_enum_variant,
    clippy::result_large_err,
    clippy::too_many_arguments,
    clippy::type_complexity
)]

//! ## Features
//!
//! - **OpenAPI parity** — full REST operation coverage for the current docs snapshot
//! - **AsyncAPI parity** — WebSocket commands, responses, and `user_orders`
//! - **Pagination helpers** — page-level ([`CursorPager`]) and item-level (`stream_*`) iteration
//! - **REST reliability controls** — retry/backoff/jitter with `429 Retry-After` support
//! - **Transport builder** — timeout/connect-timeout/headers/user-agent/proxy/custom client
//! - **RSA-PSS authentication** — secure signing for private endpoints
//!
//! ## Quick Start: REST (Builder + Retry)
//!
//! ```no_run
//! use std::time::Duration;
//! use kalshi_fast::{
//!     KalshiEnvironment, KalshiRestClient, RateLimitConfig, RetryConfig,
//! };
//!
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! let client = KalshiRestClient::builder(KalshiEnvironment::demo())
//!     .with_rate_limit_config(RateLimitConfig { read_rps: 30, write_rps: 15 })
//!     .with_retry_config(RetryConfig {
//!         max_retries: 4,
//!         base_delay: Duration::from_millis(200),
//!         max_delay: Duration::from_secs(2),
//!         jitter: 0.2,
//!         retry_non_idempotent: false,
//!     })
//!     .build()?;
//!
//! let status = client.get_exchange_status().await?;
//! println!("exchange_active={}", status.exchange_active);
//! # Ok(())
//! # }
//! ```
//!
//! ## Quick Start: WebSocket
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
//!     channels: vec![WsChannel::UserOrders],
//!     ..Default::default()
//! }).await?;
//!
//! loop {
//!     match ws.next_event().await? {
//!         WsEvent::Message(WsMessage::Data(WsDataMessage::UserOrder { msg, .. })) => {
//!             println!("order={} status={:?}", msg.order_id, msg.status);
//!         }
//!         WsEvent::Reconnected { attempt } => println!("Reconnected (attempt {})", attempt),
//!         WsEvent::Disconnected { .. } => break,
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Authentication
//!
//! Private endpoints (portfolio, orders, WebSocket fills) require RSA-PSS signing.
//! Load your key with [`KalshiAuth::from_pem_file`] or [`KalshiAuth::from_pem_str`]:
//!
//! ```no_run
//! # use kalshi_fast::{KalshiAuth, KalshiError};
//! # fn run() -> Result<(), KalshiError> {
//! // From a .key file on disk
//! let auth = KalshiAuth::from_pem_file("your-key-id", "/path/to/private.key")?;
//!
//! // Or from PEM content directly (supports PKCS#8 and PKCS#1)
//! let pem = std::fs::read_to_string("/path/to/private.key").unwrap();
//! let auth = KalshiAuth::from_pem_str("your-key-id", &pem)?;
//! # Ok(())
//! # }
//! ```
//!
//! Environment variables used by the examples:
//! - `KALSHI_KEY_ID` — your API key ID
//! - `KALSHI_PRIVATE_KEY_PATH` — path to your RSA private key (PEM format)
//!
//! ## Pagination
//!
//! **Page-level** with [`CursorPager`]:
//!
//! ```no_run
//! # use kalshi_fast::{GetMarketsParams, KalshiEnvironment, KalshiRestClient};
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! # let client = KalshiRestClient::new(KalshiEnvironment::demo());
//! let mut pager = client.markets_pager(GetMarketsParams::default());
//! while let Some(page) = pager.next_page().await? {
//!     for market in page {
//!         println!("{}", market.ticker);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! **Item-level** with streams:
//!
//! ```no_run
//! use futures::stream::TryStreamExt;
//! # use kalshi_fast::{GetMarketsParams, KalshiEnvironment, KalshiRestClient, Market};
//!
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! # let client = KalshiRestClient::new(KalshiEnvironment::demo());
//! let markets: Vec<Market> = client
//!     .stream_markets(GetMarketsParams::default(), Some(250))
//!     .try_collect()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## WebSocket Reconnection
//!
//! [`KalshiWsClient`] handles reconnection automatically with exponential backoff
//! and resubscribes to active channels. Configure via [`WsReconnectConfig`]:
//!
//! | Field | Default | Description |
//! |---|---|---|
//! | `max_retries` | `None` (unlimited) | Maximum reconnection attempts |
//! | `base_delay` | 250 ms | First backoff delay |
//! | `max_delay` | 30 s | Upper bound on backoff |
//! | `jitter` | 0.2 | Random jitter factor |
//! | `resubscribe` | `true` | Resubscribe to active channels on reconnect |
//!
//! Connection lifecycle events are exposed through [`WsEvent`]:
//!
//! - [`WsEvent::Message`] — incoming data
//! - [`WsEvent::Reconnected`] — connection restored after a drop
//! - [`WsEvent::Disconnected`] — connection lost after max retries
//!
//! **Note:** Sequence resync is not automatic; callers must handle any gaps.
//!
//! ## Performance
//!
//! Optimized for low-latency algorithmic trading:
//!
//! - **Deferred JSON parsing** — uses `serde_json::RawValue` to skip parsing unused fields
//! - **Zero-copy message parsing** — binary WebSocket frames parsed with `from_slice`
//! - **Split read/write streams** — no lock contention on WebSocket operations

pub mod auth;
pub mod env;
pub mod error;
pub mod rest;
pub mod types;
pub mod ws;

// Primary clients
pub use auth::{KalshiAuth, KalshiAuthHeaders};
pub use env::{KalshiEnvironment, REST_PREFIX, WS_PATH};
pub use error::KalshiError;
pub use rest::{
    CursorPager, KalshiRestClient, KalshiRestClientBuilder, RateLimitConfig, RateLimitTier,
    RetryConfig,
};
pub use ws::{
    KalshiWsClient, KalshiWsLowLevelClient, WsEvent, WsEventReceiver, WsReaderConfig, WsReaderMode,
    WsReconnectConfig,
};

// Backwards-compatible type re-exports
pub use rest::types::*;
pub use types::*;
pub use ws::types::*;
