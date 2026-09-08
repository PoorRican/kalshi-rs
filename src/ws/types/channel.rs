use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsChannelV2 {
    Ticker,
    Trade,
    MarketLifecycleV2,
    MultivariateMarketLifecycle,
    OrderbookDelta,
    Fill,
    MarketPositions,
    Communications,
    OrderGroupUpdates,
    UserOrders,
    /// CF Benchmarks reference index value feed. Added 2026-06-08 (AsyncAPI 2.0.0).
    CfbenchmarksValue,
}

impl WsChannelV2 {
    pub fn as_str(self) -> &'static str {
        match self {
            WsChannelV2::Ticker => "ticker",
            WsChannelV2::Trade => "trade",
            WsChannelV2::MarketLifecycleV2 => "market_lifecycle_v2",
            WsChannelV2::MultivariateMarketLifecycle => "multivariate_market_lifecycle",
            WsChannelV2::OrderbookDelta => "orderbook_delta",
            WsChannelV2::Fill => "fill",
            WsChannelV2::MarketPositions => "market_positions",
            WsChannelV2::Communications => "communications",
            WsChannelV2::OrderGroupUpdates => "order_group_updates",
            WsChannelV2::UserOrders => "user_orders",
            WsChannelV2::CfbenchmarksValue => "cfbenchmarks_value",
        }
    }

    pub fn is_private(self) -> bool {
        matches!(
            self,
            WsChannelV2::OrderbookDelta
                | WsChannelV2::Fill
                | WsChannelV2::MarketPositions
                | WsChannelV2::Communications
                | WsChannelV2::OrderGroupUpdates
                | WsChannelV2::UserOrders
        )
    }
}

impl fmt::Display for WsChannelV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_channel_check() {
        assert!(WsChannelV2::Fill.is_private());
        assert!(WsChannelV2::OrderbookDelta.is_private());
        assert!(WsChannelV2::MarketPositions.is_private());
        assert!(WsChannelV2::Communications.is_private());
        assert!(WsChannelV2::OrderGroupUpdates.is_private());

        assert!(!WsChannelV2::Ticker.is_private());
        assert!(!WsChannelV2::Trade.is_private());
        assert!(!WsChannelV2::MarketLifecycleV2.is_private());
    }
}
