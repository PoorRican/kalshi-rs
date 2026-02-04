//! Unit tests for WebSocket message parsing.

use kalshi::{WsDataMessage, WsEnvelope, WsMessage, WsOrderbookDelta, WsTicker};

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
    assert_eq!(env.msg_type, "orderbook_snapshot");
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
