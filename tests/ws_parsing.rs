//! Unit tests for WebSocket message parsing.

use kalshi::{
    MarketStatus, WsCommunications, WsDataMessage, WsEnvelope, WsMessage, WsMsgType,
    WsOrderGroupEventType, WsOrderbookDelta, WsTicker, YesNo,
};

#[test]
fn ws_envelope_deserializes_with_sid_and_seq() {
    let json = r#"{
        "id": 1,
        "type": "orderbook_snapshot",
        "sid": 42,
        "seq": 100,
        "msg": {"market_ticker": "TEST", "market_id": "abc123"}
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(env.id, Some(1));
    assert_eq!(env.msg_type, WsMsgType::OrderbookSnapshot);
    assert_eq!(env.sid, Some(42));
    assert_eq!(env.seq, Some(100));
    assert!(env.msg.is_some());
}

#[test]
fn ws_message_subscribed_parses() {
    let json = r#"{
        "id": 5,
        "type": "subscribed",
        "sid": 99
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Subscribed { id, sid } => {
            assert_eq!(id, Some(5));
            assert_eq!(sid, Some(99));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_message_list_subscriptions_parses() {
    let json = r#"{
        "id": 7,
        "type": "list_subscriptions",
        "msg": {
            "subscriptions": [
                {"sid": 1, "channels": ["ticker"], "market_tickers": ["TEST"]}
            ]
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::ListSubscriptions { id, subscriptions } => {
            assert_eq!(id, Some(7));
            assert_eq!(subscriptions.len(), 1);
            assert_eq!(subscriptions[0].sid, 1);
            assert_eq!(subscriptions[0].market_tickers.as_ref().unwrap()[0], "TEST");
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_message_error_parses() {
    let json = r#"{
        "id": 9,
        "type": "error",
        "msg": {"code": 9, "message": "Authentication required"}
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Error { id, error } => {
            assert_eq!(id, Some(9));
            assert_eq!(error.code, Some(9));
            assert_eq!(error.message.as_deref(), Some("Authentication required"));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_ticker_message_parses() {
    let json = r#"{
        "type": "ticker",
        "msg": {
            "market_ticker": "INXD-25JAN10-T17900",
            "market_id": "abc123",
            "price": 55,
            "yes_bid": 54,
            "yes_ask": 56,
            "price_dollars": "0.55",
            "yes_bid_dollars": "0.54",
            "yes_ask_dollars": "0.56",
            "volume": 10000,
            "volume_fp": "10000.00",
            "open_interest": 5000,
            "open_interest_fp": "5000.00",
            "dollar_volume": 5500,
            "dollar_open_interest": 2750,
            "ts": 1700000000000
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Ticker { msg, .. }) => {
            assert_eq!(msg.market_ticker, "INXD-25JAN10-T17900");
            assert_eq!(msg.price, 55);
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_ticker_v2_message_parses() {
    let json = r#"{
        "type": "ticker_v2",
        "msg": {
            "market_ticker": "INXD-25JAN10-T17900",
            "price": 55,
            "ts": 1700000000000
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::TickerV2 { msg, .. }) => {
            assert_eq!(msg.market_ticker, "INXD-25JAN10-T17900");
            assert_eq!(msg.price, Some(55));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_trade_message_parses() {
    let json = r#"{
        "type": "trade",
        "msg": {
            "trade_id": "trade-1",
            "ticker": "MKT-1",
            "price": 55,
            "count": 10,
            "taker_side": "yes",
            "created_time": "2025-01-10T12:00:00Z"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Trade { msg, .. }) => {
            assert_eq!(msg.trade_id, "trade-1");
            assert_eq!(msg.ticker, "MKT-1");
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_orderbook_snapshot_deserializes() {
    let json = r#"{
        "type": "orderbook_snapshot",
        "msg": {
            "market_ticker": "INXD-25JAN10-T17900",
            "market_id": "abc123",
            "yes": [[50, 100], [51, 200]],
            "yes_dollars": [["0.50", 100], ["0.51", 200]],
            "no": [[49, 150]],
            "no_dollars": [["0.49", 150]]
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::OrderbookSnapshot { msg, .. }) => {
            assert_eq!(msg.market_ticker, "INXD-25JAN10-T17900");
            assert_eq!(msg.yes.len(), 2);
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_orderbook_delta_deserializes() {
    let json = r#"{
        "type": "orderbook_delta",
        "msg": {
            "market_ticker": "INXD-25JAN10-T17900",
            "market_id": "abc123",
            "price": 55,
            "price_dollars": "0.55",
            "delta": 50,
            "delta_fp": "50.00",
            "side": "yes",
            "client_order_id": "my-order-1",
            "subaccount": 0,
            "ts": "2025-01-10T12:00:00Z"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::OrderbookDelta { msg, .. }) => {
            assert_eq!(msg.market_ticker, "INXD-25JAN10-T17900");
            assert_eq!(msg.delta, 50);
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_fill_deserializes() {
    let json = r#"{
        "type": "fill",
        "msg": {
            "fill_id": "fill-123",
            "trade_id": "trade-456",
            "order_id": "order-789",
            "client_order_id": "my-order",
            "ticker": "INXD-25JAN10-T17900",
            "market_ticker": "INXD-25JAN10-T17900",
            "side": "yes",
            "action": "buy",
            "count": 10,
            "count_fp": "10.00",
            "yes_price": 55,
            "no_price": 45,
            "yes_price_fixed": "0.55",
            "no_price_fixed": "0.45",
            "is_taker": true,
            "fee_cost": "0.05",
            "created_time": "2025-01-10T12:00:00Z",
            "subaccount_number": 1,
            "ts": 1700000000000
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Fill { msg, .. }) => {
            assert_eq!(msg.fill_id, "fill-123");
            assert_eq!(msg.trade_id, "trade-456");
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_envelope_parse_ticker_raw() {
    let json = r#"{
        "type": "ticker",
        "sid": 1,
        "msg": {
            "market_ticker": "TEST",
            "market_id": "abc",
            "price": 50,
            "yes_bid": 49,
            "yes_ask": 51,
            "price_dollars": "0.50",
            "yes_bid_dollars": "0.49",
            "yes_ask_dollars": "0.51",
            "volume": 1000,
            "volume_fp": "1000.00",
            "open_interest": 500,
            "open_interest_fp": "500.00",
            "dollar_volume": 500,
            "dollar_open_interest": 250,
            "ts": 1700000000
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let raw = env.msg_raw().unwrap();
    let msg: WsTicker = serde_json::from_str(raw).unwrap();
    assert_eq!(msg.market_ticker, "TEST");
}

#[test]
fn ws_orderbook_delta_raw() {
    let json = r#"{
        "type": "orderbook_delta",
        "msg": {
            "market_ticker": "TEST",
            "market_id": "abc",
            "price": 50,
            "price_dollars": "0.50",
            "delta": 10,
            "delta_fp": "10.00",
            "side": "yes"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let raw = env.msg_raw().unwrap();
    let msg: WsOrderbookDelta = serde_json::from_str(raw).unwrap();
    assert_eq!(msg.delta, 10);
}

#[test]
fn ws_market_lifecycle_v2_message_parses() {
    let json = r#"{
        "type": "market_lifecycle_v2",
        "msg": {
            "market_ticker": "MKT-1",
            "status": "open",
            "can_trade": true
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::MarketLifecycleV2 { msg, .. }) => {
            assert_eq!(msg.market_ticker, "MKT-1");
            assert_eq!(msg.status, Some(MarketStatus::Open));
            assert_eq!(msg.can_trade, Some(true));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_market_positions_message_parses() {
    let json = r#"{
        "type": "market_positions",
        "msg": {
            "market_positions": [{"ticker":"MKT-1","position":1}],
            "event_positions": [{"event_ticker":"EVT-1","position":2}]
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::MarketPositions { msg, .. }) => {
            assert_eq!(msg.market_positions.len(), 1);
            assert_eq!(msg.market_positions[0].ticker, "MKT-1");
            assert_eq!(msg.event_positions[0].event_ticker, "EVT-1");
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_rfq_created_message_parses() {
    let json = r#"{
        "type": "rfq_created",
        "msg": {
            "id": "rfq_123",
            "creator_id": "",
            "market_ticker": "FED-23DEC-T3.00",
            "created_ts": "2024-12-01T10:00:00Z",
            "mve_selected_legs": [
                {"event_ticker":"EVT-1","market_ticker":"MKT-1","side":"yes"}
            ]
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Communications { msg, .. }) => {
            match msg {
                WsCommunications::RfqCreated(rfq) => {
                    assert_eq!(rfq.id, "rfq_123");
                    assert_eq!(rfq.market_ticker, "FED-23DEC-T3.00");
                    assert!(matches!(
                        rfq.mve_selected_legs.as_ref().unwrap()[0].side,
                        Some(YesNo::Yes)
                    ));
                }
                other => panic!("unexpected communications payload: {:?}", other),
            }
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_rfq_deleted_message_parses() {
    let json = r#"{
        "type": "rfq_deleted",
        "msg": {
            "id": "rfq_124",
            "creator_id": "creator",
            "market_ticker": "FED-23DEC-T3.00",
            "deleted_ts": "2024-12-01T10:05:00Z"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Communications { msg, .. }) => {
            match msg {
                WsCommunications::RfqDeleted(rfq) => {
                    assert_eq!(rfq.id, "rfq_124");
                    assert_eq!(rfq.creator_id, "creator");
                }
                other => panic!("unexpected communications payload: {:?}", other),
            }
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_quote_created_message_parses() {
    let json = r#"{
        "type": "quote_created",
        "msg": {
            "quote_id": "q-1",
            "rfq_id": "rfq-1",
            "quote_creator_id": "creator",
            "market_ticker": "FED-23DEC-T3.00",
            "yes_bid": 50,
            "no_bid": 50,
            "yes_bid_dollars": "0.50",
            "no_bid_dollars": "0.50",
            "created_ts": "2024-12-01T10:06:00Z"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Communications { msg, .. }) => {
            match msg {
                WsCommunications::QuoteCreated(quote) => {
                    assert_eq!(quote.quote_id, "q-1");
                    assert_eq!(quote.yes_bid, 50);
                }
                other => panic!("unexpected communications payload: {:?}", other),
            }
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_quote_accepted_message_parses() {
    let json = r#"{
        "type": "quote_accepted",
        "msg": {
            "quote_id": "q-2",
            "rfq_id": "rfq-2",
            "quote_creator_id": "creator",
            "market_ticker": "FED-23DEC-T3.00",
            "yes_bid": 51,
            "no_bid": 49,
            "yes_bid_dollars": "0.51",
            "no_bid_dollars": "0.49",
            "accepted_side": "yes",
            "contracts_accepted": 10
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Communications { msg, .. }) => {
            match msg {
                WsCommunications::QuoteAccepted(quote) => {
                    assert_eq!(quote.quote_id, "q-2");
                    assert!(matches!(quote.accepted_side, Some(YesNo::Yes)));
                    assert_eq!(quote.contracts_accepted, Some(10));
                }
                other => panic!("unexpected communications payload: {:?}", other),
            }
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_quote_executed_message_parses() {
    let json = r#"{
        "type": "quote_executed",
        "msg": {
            "quote_id": "q-3",
            "rfq_id": "rfq-3",
            "quote_creator_id": "creator",
            "rfq_creator_id": "rfq_creator",
            "order_id": "order-1",
            "client_order_id": "client-1",
            "market_ticker": "FED-23DEC-T3.00",
            "executed_ts": "2024-12-01T10:07:00Z"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Communications { msg, .. }) => {
            match msg {
                WsCommunications::QuoteExecuted(quote) => {
                    assert_eq!(quote.quote_id, "q-3");
                    assert_eq!(quote.order_id, "order-1");
                }
                other => panic!("unexpected communications payload: {:?}", other),
            }
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_multivariate_message_parses() {
    let json = r#"{
        "type": "multivariate_lookup",
        "msg": {
            "collection_ticker": "COLL-1",
            "event_ticker": "EVT-1",
            "market_ticker": "MKT-1",
            "selected_markets": [
                {"event_ticker":"EVT-1","market_ticker":"MKT-1","side":"yes"}
            ]
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::Multivariate { msg, .. }) => {
            assert_eq!(msg.collection_ticker, "COLL-1");
            assert!(matches!(msg.selected_markets[0].side, YesNo::Yes));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_order_group_updates_message_parses() {
    let json = r#"{
        "type": "order_group_updates",
        "msg": {"event_type":"limit_updated","order_group_id":"og-1","contracts_limit_fp":"150.00"}
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::Data(WsDataMessage::OrderGroupUpdates { msg, .. }) => {
            assert_eq!(msg.order_group_id, "og-1");
            assert!(matches!(msg.event_type, WsOrderGroupEventType::LimitUpdated));
            assert_eq!(msg.contracts_limit_fp.as_deref(), Some("150.00"));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn ws_list_subscriptions_parses_from_subscriptions_field() {
    let json = r#"{
        "id": 2,
        "type": "list_subscriptions",
        "subscriptions": [
            {"sid": 10, "channels": ["ticker"], "market_tickers": ["TEST"]}
        ]
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let msg = env.into_message().unwrap();
    match msg {
        WsMessage::ListSubscriptions { id, subscriptions } => {
            assert_eq!(id, Some(2));
            assert_eq!(subscriptions.len(), 1);
            assert_eq!(subscriptions[0].sid, 10);
        }
        other => panic!("unexpected: {:?}", other),
    }
}
