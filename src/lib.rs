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
pub use rest::{CursorPager, KalshiRestClient, RateLimitConfig, RateLimitTier};
pub use ws::{KalshiWsClient, KalshiWsLowLevelClient, WsEvent, WsReconnectConfig};

// Backwards-compatible type re-exports
pub use types::*;
pub use rest::types::*;
pub use ws::types::*;
