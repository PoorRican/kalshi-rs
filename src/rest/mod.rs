//! REST client for the Kalshi API.
//!
//! [`KalshiRestClient`] wraps all public and authenticated HTTP endpoints.
//! Public endpoints (markets, events, trades, exchange status) need no auth;
//! portfolio endpoints (orders, fills, positions, settlements) require a
//! [`KalshiAuth`](crate::KalshiAuth) attached via [`KalshiRestClient::with_auth`].
//!
//! # Rate Limiting
//!
//! Every request passes through a built-in rate limiter that enforces separate
//! read (GET) and write (POST/DELETE) budgets. The default is the Basic tier
//! (10 write RPS / 20 read RPS). Override with [`KalshiRestClient::with_rate_limit_config`].
//!
//! # Pagination
//!
//! Endpoints that return lists use cursor-based pagination. Three styles are
//! available for each paginated resource:
//!
//! | Style | Method | Yields |
//! |-------|--------|--------|
//! | Single page | `get_markets(params)` | One page per call |
//! | Page iterator | `markets_pager(params)` | `Vec<Market>` per `next_page()` |
//! | Item stream | `stream_markets(params, max)` | One `Market` at a time |
//! | Bulk collect | `get_markets_all(params)` | All items in a single `Vec` |
//!
//! See [`CursorPager`] for page-level control, the `stream_*` methods for
//! item-level async iteration, and the `get_*_all` methods for eagerly
//! collecting every page into memory.
//!
//! # Example
//!
//! ```no_run
//! use kalshi_fast::{
//!     GetMarketsParams, KalshiAuth, KalshiEnvironment,
//!     KalshiRestClient, MarketStatus,
//! };
//!
//! # async fn run() -> Result<(), kalshi_fast::KalshiError> {
//! let auth = KalshiAuth::from_pem_file("key-id", "/path/to/key.pem")?;
//!
//! let client = KalshiRestClient::new(KalshiEnvironment::demo())
//!     .with_auth(auth);
//!
//! // Public endpoint — no auth needed
//! let resp = client
//!     .get_markets(GetMarketsParams {
//!         status: Some(MarketStatus::Open),
//!         limit: Some(5),
//!         ..Default::default()
//!     })
//!     .await?;
//!
//! // Authenticated endpoint
//! let balance = client.get_balance().await?;
//! println!("balance: {}", balance.balance);
//! # Ok(())
//! # }
//! ```

mod client;
pub mod types;

pub use client::{CursorPager, KalshiRestClient, RateLimitConfig, RateLimitTier};
pub use types::*;
