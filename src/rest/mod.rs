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
//!     GetBalanceParams, GetMarketsParams, KalshiAuth, KalshiEnvironment,
//!     KalshiRestClient, MarketStatusQuery,
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
//!         status: Some(MarketStatusQuery::Open),
//!         limit: Some(5),
//!         ..Default::default()
//!     })
//!     .await?;
//!
//! // Authenticated endpoint
//! let balance = client.get_balance(GetBalanceParams::default()).await?;
//! println!("balance: {}", balance.balance);
//! # Ok(())
//! # }
//! ```

mod account;
mod client;
mod communications;
mod events;
mod exchange;
mod live_data;
mod markets;
mod multivariate;
mod orders;
mod pagination;
mod portfolio;
mod rate_limit;
mod retry;
mod series;
mod structured_targets;
mod trades;

pub use account::*;
pub use client::{KalshiRestClient, KalshiRestClientBuilder};
pub use communications::*;
pub use events::*;
pub use exchange::*;
pub use live_data::*;
pub use markets::*;
pub use multivariate::*;
pub use orders::*;
pub use pagination::CursorPager;
pub use portfolio::*;
pub use rate_limit::{RateLimitConfig, RateLimitTier};
pub use retry::RetryConfig;
pub use series::*;
pub use structured_targets::*;
pub use trades::*;
