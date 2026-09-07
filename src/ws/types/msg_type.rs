use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WsMsgType {
    Subscribed,
    Unsubscribed,
    Ok,
    ListSubscriptions,
    Error,
    Ticker,
    Trade,
    OrderbookSnapshot,
    OrderbookDelta,
    Fill,
    MarketPosition,
    MarketLifecycleV2,
    MultivariateMarketLifecycle,
    EventLifecycle,
    EventFeeUpdate,
    Communications,
    RfqCreated,
    RfqDeleted,
    QuoteCreated,
    QuoteAccepted,
    QuoteExecuted,
    OrderGroupUpdates,
    UserOrder,
    CfbenchmarksValue,
    CfbenchmarksValueIndexlist,
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
            WsMsgType::Trade => "trade",
            WsMsgType::OrderbookSnapshot => "orderbook_snapshot",
            WsMsgType::OrderbookDelta => "orderbook_delta",
            WsMsgType::Fill => "fill",
            WsMsgType::MarketPosition => "market_position",
            WsMsgType::MarketLifecycleV2 => "market_lifecycle_v2",
            WsMsgType::MultivariateMarketLifecycle => "multivariate_market_lifecycle",
            WsMsgType::EventLifecycle => "event_lifecycle",
            WsMsgType::EventFeeUpdate => "event_fee_update",
            WsMsgType::Communications => "communications",
            WsMsgType::RfqCreated => "rfq_created",
            WsMsgType::RfqDeleted => "rfq_deleted",
            WsMsgType::QuoteCreated => "quote_created",
            WsMsgType::QuoteAccepted => "quote_accepted",
            WsMsgType::QuoteExecuted => "quote_executed",
            WsMsgType::OrderGroupUpdates => "order_group_updates",
            WsMsgType::UserOrder => "user_order",
            WsMsgType::CfbenchmarksValue => "cfbenchmarks_value",
            WsMsgType::CfbenchmarksValueIndexlist => "cfbenchmarks_value_indexlist",
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
            "trade" => WsMsgType::Trade,
            "orderbook_snapshot" => WsMsgType::OrderbookSnapshot,
            "orderbook_delta" => WsMsgType::OrderbookDelta,
            "fill" => WsMsgType::Fill,
            "market_position" | "market_positions" => WsMsgType::MarketPosition,
            "market_lifecycle_v2" => WsMsgType::MarketLifecycleV2,
            "multivariate_market_lifecycle" => WsMsgType::MultivariateMarketLifecycle,
            "event_lifecycle" | "event_lifecycle_v2" => WsMsgType::EventLifecycle,
            "event_fee_update" => WsMsgType::EventFeeUpdate,
            "communications" => WsMsgType::Communications,
            "rfq_created" => WsMsgType::RfqCreated,
            "rfq_deleted" => WsMsgType::RfqDeleted,
            "quote_created" => WsMsgType::QuoteCreated,
            "quote_accepted" => WsMsgType::QuoteAccepted,
            "quote_executed" => WsMsgType::QuoteExecuted,
            "order_group_updates" => WsMsgType::OrderGroupUpdates,
            "user_order" => WsMsgType::UserOrder,
            "cfbenchmarks_value" => WsMsgType::CfbenchmarksValue,
            "cfbenchmarks_value_indexlist" => WsMsgType::CfbenchmarksValueIndexlist,
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
            "trade" => WsMsgType::Trade,
            "orderbook_snapshot" => WsMsgType::OrderbookSnapshot,
            "orderbook_delta" => WsMsgType::OrderbookDelta,
            "fill" => WsMsgType::Fill,
            "market_position" | "market_positions" => WsMsgType::MarketPosition,
            "market_lifecycle_v2" => WsMsgType::MarketLifecycleV2,
            "multivariate_market_lifecycle" => WsMsgType::MultivariateMarketLifecycle,
            "event_lifecycle" | "event_lifecycle_v2" => WsMsgType::EventLifecycle,
            "event_fee_update" => WsMsgType::EventFeeUpdate,
            "communications" => WsMsgType::Communications,
            "rfq_created" => WsMsgType::RfqCreated,
            "rfq_deleted" => WsMsgType::RfqDeleted,
            "quote_created" => WsMsgType::QuoteCreated,
            "quote_accepted" => WsMsgType::QuoteAccepted,
            "quote_executed" => WsMsgType::QuoteExecuted,
            "order_group_updates" => WsMsgType::OrderGroupUpdates,
            "user_order" => WsMsgType::UserOrder,
            "cfbenchmarks_value" => WsMsgType::CfbenchmarksValue,
            "cfbenchmarks_value_indexlist" => WsMsgType::CfbenchmarksValueIndexlist,
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

            fn visit_borrowed_str<E: serde::de::Error>(
                self,
                value: &'de str,
            ) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_str(value)
                    .unwrap_or_else(|| WsMsgType::Unknown(value.to_owned())))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_str(value)
                    .unwrap_or_else(|| WsMsgType::Unknown(value.to_owned())))
            }

            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(WsMsgType::from_string(value))
            }
        }

        deserializer.deserialize_str(WsMsgTypeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
