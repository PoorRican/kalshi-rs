use super::channel::WsChannelV2;
use super::envelope::{
    WsDataMessageRef, WsDataMessageV2, WsError, WsErrorRef, WsListSubscriptions,
    WsListSubscriptionsRef, WsMessageRef, WsMessageV2,
};
use super::messages::*;
use super::subscription::{WsSubscriptionInfo, WsSubscriptionInfoRef};
use serde::Deserialize;
use serde_json::Value;
use serde_json::value::RawValue;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(super) enum WsWireMessage {
    #[serde(rename = "subscribed")]
    Subscribed {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default)]
        msg: Option<WsSubscribedMsg>,
    },
    #[serde(rename = "unsubscribed")]
    Unsubscribed {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
    },
    #[serde(rename = "ok")]
    Ok {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default)]
        msg: Option<Value>,
    },
    #[serde(rename = "list_subscriptions")]
    ListSubscriptions {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default)]
        subscriptions: Vec<WsSubscriptionInfo>,
        #[serde(default)]
        msg: Option<WsListSubscriptions>,
    },
    #[serde(rename = "error")]
    Error {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default)]
        msg: Option<WsError>,
    },
    #[serde(rename = "ticker")]
    Ticker {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTicker,
    },
    #[serde(rename = "trade")]
    Trade {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsTrade,
    },
    #[serde(rename = "orderbook_snapshot")]
    OrderbookSnapshot {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookSnapshot,
    },
    #[serde(rename = "orderbook_delta")]
    OrderbookDelta {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderbookDelta,
    },
    #[serde(rename = "fill")]
    Fill {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsFill,
    },
    #[serde(rename = "market_position", alias = "market_positions")]
    MarketPosition {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketPosition,
    },
    #[serde(rename = "market_lifecycle_v2")]
    MarketLifecycleV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketLifecycleV2,
    },
    #[serde(rename = "multivariate_market_lifecycle")]
    MultivariateMarketLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsMarketLifecycleV2,
    },
    #[serde(rename = "event_lifecycle", alias = "event_lifecycle_v2")]
    EventLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsEventLifecycle,
    },
    #[serde(rename = "event_fee_update")]
    EventFeeUpdate {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsEventFeeUpdate,
    },
    #[serde(rename = "rfq_created")]
    RfqCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsRfqCreated,
    },
    #[serde(rename = "rfq_deleted")]
    RfqDeleted {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsRfqDeleted,
    },
    #[serde(rename = "quote_created")]
    QuoteCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsQuoteCreated,
    },
    #[serde(rename = "quote_accepted")]
    QuoteAccepted {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsQuoteAccepted,
    },
    #[serde(rename = "quote_executed")]
    QuoteExecuted {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsQuoteExecuted,
    },
    #[serde(rename = "order_group_updates")]
    OrderGroupUpdates {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsOrderGroupUpdate,
    },
    #[serde(rename = "user_order")]
    UserOrder {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsUserOrder,
    },
    #[serde(rename = "cfbenchmarks_value")]
    CfbenchmarksValue {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsCfBenchmarksValue,
    },
    #[serde(rename = "cfbenchmarks_value_indexlist")]
    CfbenchmarksValueIndexlist {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsCfBenchmarksIndexList,
    },
}

#[derive(Debug, Deserialize)]
pub(super) struct WsSubscribedMsg {
    #[allow(dead_code)]
    pub channel: Option<WsChannelV2>,
    pub sid: Option<u64>,
}

impl WsWireMessage {
    pub(super) fn into_message(self) -> WsMessageV2 {
        match self {
            WsWireMessage::Subscribed { id, sid, seq, msg } => WsMessageV2::Subscribed {
                id,
                sid: sid.or_else(|| msg.and_then(|value| value.sid)),
                seq,
            },
            WsWireMessage::Unsubscribed { id, sid, seq } => {
                WsMessageV2::Unsubscribed { id, sid, seq }
            }
            WsWireMessage::Ok { id, sid, seq, msg } => {
                if let Some(msg) = msg
                    && let Ok(subscriptions) =
                        serde_json::from_value::<Vec<WsSubscriptionInfo>>(msg)
                {
                    return WsMessageV2::ListSubscriptions {
                        id,
                        sid,
                        seq,
                        subscriptions,
                    };
                }
                WsMessageV2::Ok { id, sid, seq }
            }
            WsWireMessage::ListSubscriptions {
                id,
                sid,
                seq,
                subscriptions,
                msg,
            } => {
                let subs = msg
                    .map(|value| value.subscriptions)
                    .unwrap_or(subscriptions);
                WsMessageV2::ListSubscriptions {
                    id,
                    sid,
                    seq,
                    subscriptions: subs,
                }
            }
            WsWireMessage::Error { id, sid, seq, msg } => WsMessageV2::Error {
                id,
                sid,
                seq,
                error: msg.unwrap_or(WsError {
                    code: None,
                    message: None,
                }),
            },
            WsWireMessage::Ticker { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Ticker { sid, seq, msg })
            }
            WsWireMessage::Trade { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Trade { sid, seq, msg })
            }
            WsWireMessage::OrderbookSnapshot { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::OrderbookSnapshot { sid, seq, msg })
            }
            WsWireMessage::OrderbookDelta { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::OrderbookDelta { sid, seq, msg })
            }
            WsWireMessage::Fill { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Fill { sid, seq, msg })
            }
            WsWireMessage::MarketPosition { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::MarketPosition { sid, seq, msg })
            }
            WsWireMessage::MarketLifecycleV2 { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::MarketLifecycleV2 { sid, seq, msg })
            }
            WsWireMessage::MultivariateMarketLifecycle { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::MultivariateMarketLifecycle { sid, seq, msg })
            }
            WsWireMessage::EventLifecycle { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::EventLifecycle { sid, seq, msg })
            }
            WsWireMessage::EventFeeUpdate { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::EventFeeUpdate { sid, seq, msg })
            }
            WsWireMessage::RfqCreated { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::RfqCreated(msg),
                })
            }
            WsWireMessage::RfqDeleted { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::RfqDeleted(msg),
                })
            }
            WsWireMessage::QuoteCreated { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::QuoteCreated(msg),
                })
            }
            WsWireMessage::QuoteAccepted { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::QuoteAccepted(msg),
                })
            }
            WsWireMessage::QuoteExecuted { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::Communications {
                    sid,
                    seq,
                    msg: WsCommunications::QuoteExecuted(msg),
                })
            }
            WsWireMessage::OrderGroupUpdates { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::OrderGroupUpdates { sid, seq, msg })
            }
            WsWireMessage::UserOrder { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::UserOrder { sid, seq, msg })
            }
            WsWireMessage::CfbenchmarksValue { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::CfbenchmarksValue { sid, seq, msg })
            }
            WsWireMessage::CfbenchmarksValueIndexlist { sid, seq, msg } => {
                WsMessageV2::Data(WsDataMessageV2::CfbenchmarksValueIndexlist { sid, seq, msg })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(super) enum WsWireMessageRef<'a> {
    #[serde(rename = "subscribed")]
    Subscribed {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default)]
        msg: Option<WsSubscribedMsgRef>,
    },
    #[serde(rename = "unsubscribed")]
    Unsubscribed {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
    },
    #[serde(rename = "ok")]
    Ok {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default, borrow)]
        msg: Option<&'a RawValue>,
    },
    #[serde(rename = "list_subscriptions")]
    ListSubscriptions {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default, borrow)]
        subscriptions: Vec<WsSubscriptionInfoRef<'a>>,
        #[serde(default, borrow)]
        msg: Option<WsListSubscriptionsRef<'a>>,
    },
    #[serde(rename = "error")]
    Error {
        id: Option<u64>,
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(default, borrow)]
        msg: Option<WsErrorRef<'a>>,
    },
    #[serde(rename = "ticker")]
    Ticker {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsTickerRef<'a>,
    },
    #[serde(rename = "trade")]
    Trade {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsTradeRef<'a>,
    },
    #[serde(rename = "orderbook_snapshot")]
    OrderbookSnapshot {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsOrderbookSnapshotRef<'a>,
    },
    #[serde(rename = "orderbook_delta")]
    OrderbookDelta {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsOrderbookDeltaRef<'a>,
    },
    #[serde(rename = "fill")]
    Fill {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsFillRef<'a>,
    },
    #[serde(rename = "market_position", alias = "market_positions")]
    MarketPosition {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMarketPositionRef<'a>,
    },
    #[serde(rename = "market_lifecycle_v2")]
    MarketLifecycleV2 {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMarketLifecycleV2Ref<'a>,
    },
    #[serde(rename = "multivariate_market_lifecycle")]
    MultivariateMarketLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsMarketLifecycleV2Ref<'a>,
    },
    #[serde(rename = "event_lifecycle", alias = "event_lifecycle_v2")]
    EventLifecycle {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsEventLifecycleRef<'a>,
    },
    #[serde(rename = "event_fee_update")]
    EventFeeUpdate {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsEventFeeUpdateRef<'a>,
    },
    #[serde(rename = "rfq_created")]
    RfqCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsRfqCreatedRef<'a>,
    },
    #[serde(rename = "rfq_deleted")]
    RfqDeleted {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsRfqDeletedRef<'a>,
    },
    #[serde(rename = "quote_created")]
    QuoteCreated {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsQuoteCreatedRef<'a>,
    },
    #[serde(rename = "quote_accepted")]
    QuoteAccepted {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsQuoteAcceptedRef<'a>,
    },
    #[serde(rename = "quote_executed")]
    QuoteExecuted {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsQuoteExecutedRef<'a>,
    },
    #[serde(rename = "order_group_updates")]
    OrderGroupUpdates {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsOrderGroupUpdateRef<'a>,
    },
    #[serde(rename = "user_order")]
    UserOrder {
        sid: Option<u64>,
        seq: Option<u64>,
        msg: WsUserOrder,
    },
    #[serde(rename = "cfbenchmarks_value")]
    CfbenchmarksValue {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsCfBenchmarksValueRef<'a>,
    },
    #[serde(rename = "cfbenchmarks_value_indexlist")]
    CfbenchmarksValueIndexlist {
        sid: Option<u64>,
        seq: Option<u64>,
        #[serde(borrow)]
        msg: WsCfBenchmarksIndexListRef<'a>,
    },
}

#[derive(Debug, Deserialize)]
pub(super) struct WsSubscribedMsgRef {
    #[allow(dead_code)]
    pub channel: Option<WsChannelV2>,
    #[serde(default)]
    pub sid: Option<u64>,
}

impl<'a> WsWireMessageRef<'a> {
    pub(super) fn into_message(self) -> WsMessageRef<'a> {
        match self {
            WsWireMessageRef::Subscribed { id, sid, seq, msg } => WsMessageRef::Subscribed {
                id,
                sid: sid.or_else(|| msg.and_then(|value| value.sid)),
                seq,
            },
            WsWireMessageRef::Unsubscribed { id, sid, seq } => {
                WsMessageRef::Unsubscribed { id, sid, seq }
            }
            WsWireMessageRef::Ok { id, sid, seq, msg } => {
                if let Some(raw) = msg
                    && let Ok(subscriptions) =
                        serde_json::from_str::<Vec<WsSubscriptionInfoRef<'a>>>(raw.get())
                {
                    return WsMessageRef::ListSubscriptions {
                        id,
                        sid,
                        seq,
                        subscriptions,
                    };
                }
                WsMessageRef::Ok { id, sid, seq }
            }
            WsWireMessageRef::ListSubscriptions {
                id,
                sid,
                seq,
                subscriptions,
                msg,
            } => {
                let subs = msg
                    .map(|value| value.subscriptions)
                    .unwrap_or(subscriptions);
                WsMessageRef::ListSubscriptions {
                    id,
                    sid,
                    seq,
                    subscriptions: subs,
                }
            }
            WsWireMessageRef::Error { id, sid, seq, msg } => WsMessageRef::Error {
                id,
                sid,
                seq,
                error: msg.unwrap_or(WsErrorRef {
                    code: None,
                    message: None,
                }),
            },
            WsWireMessageRef::Ticker { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Ticker { sid, seq, msg })
            }
            WsWireMessageRef::Trade { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Trade { sid, seq, msg })
            }
            WsWireMessageRef::OrderbookSnapshot { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::OrderbookSnapshot { sid, seq, msg })
            }
            WsWireMessageRef::OrderbookDelta { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::OrderbookDelta { sid, seq, msg })
            }
            WsWireMessageRef::Fill { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Fill { sid, seq, msg })
            }
            WsWireMessageRef::MarketPosition { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::MarketPosition { sid, seq, msg })
            }
            WsWireMessageRef::MarketLifecycleV2 { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::MarketLifecycleV2 { sid, seq, msg })
            }
            WsWireMessageRef::MultivariateMarketLifecycle { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::MultivariateMarketLifecycle { sid, seq, msg })
            }
            WsWireMessageRef::EventLifecycle { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::EventLifecycle { sid, seq, msg })
            }
            WsWireMessageRef::EventFeeUpdate { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::EventFeeUpdate { sid, seq, msg })
            }
            WsWireMessageRef::RfqCreated { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::RfqCreated(msg),
                })
            }
            WsWireMessageRef::RfqDeleted { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::RfqDeleted(msg),
                })
            }
            WsWireMessageRef::QuoteCreated { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::QuoteCreated(msg),
                })
            }
            WsWireMessageRef::QuoteAccepted { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::QuoteAccepted(msg),
                })
            }
            WsWireMessageRef::QuoteExecuted { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::Communications {
                    sid,
                    seq,
                    msg: WsCommunicationsRef::QuoteExecuted(msg),
                })
            }
            WsWireMessageRef::OrderGroupUpdates { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::OrderGroupUpdates { sid, seq, msg })
            }
            WsWireMessageRef::UserOrder { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::UserOrder { sid, seq, msg })
            }
            WsWireMessageRef::CfbenchmarksValue { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::CfbenchmarksValue { sid, seq, msg })
            }
            WsWireMessageRef::CfbenchmarksValueIndexlist { sid, seq, msg } => {
                WsMessageRef::Data(WsDataMessageRef::CfbenchmarksValueIndexlist { sid, seq, msg })
            }
        }
    }
}
