//! Unit tests for WebSocket message parsing.

use kalshi::{WsEnvelope, WsFill, WsOrderbookDelta, WsOrderbookSnapshot, WsTicker};

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
fn ws_envelope_deserializes_without_optional_fields() {
    let json = r#"{
        "type": "ticker",
        "msg": {}
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(env.id, None);
    assert_eq!(env.msg_type, "ticker");
    assert_eq!(env.sid, None);
    assert_eq!(env.seq, None);
}

#[test]
fn ws_ticker_deserializes() {
    let json = r#"{
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
    }"#;

    let ticker: WsTicker = serde_json::from_str(json).unwrap();
    assert_eq!(ticker.market_ticker, "INXD-25JAN10-T17900");
    assert_eq!(ticker.market_id, "abc123");
    assert_eq!(ticker.price, 55);
    assert_eq!(ticker.yes_bid, 54);
    assert_eq!(ticker.yes_ask, 56);
    assert_eq!(ticker.price_dollars, "0.55");
    assert_eq!(ticker.yes_bid_dollars, "0.54");
    assert_eq!(ticker.yes_ask_dollars, "0.56");
    assert_eq!(ticker.volume, 10000);
    assert_eq!(ticker.open_interest, 5000);
    assert_eq!(ticker.dollar_volume, 5500);
    assert_eq!(ticker.dollar_open_interest, 2750);
    assert_eq!(ticker.ts, 1700000000000);
}

#[test]
fn ws_orderbook_snapshot_deserializes() {
    let json = r#"{
        "market_ticker": "INXD-25JAN10-T17900",
        "market_id": "abc123",
        "yes": [[50, 100], [51, 200]],
        "yes_dollars": [["0.50", "100.00"], ["0.51", "200.00"]],
        "no": [[49, 150]],
        "no_dollars": [["0.49", "150.00"]]
    }"#;

    let snap: WsOrderbookSnapshot = serde_json::from_str(json).unwrap();
    assert_eq!(snap.market_ticker, "INXD-25JAN10-T17900");
    assert_eq!(snap.market_id, "abc123");
    assert_eq!(snap.yes.len(), 2);
    assert_eq!(snap.yes[0], (50, 100));
    assert_eq!(snap.yes[1], (51, 200));
    assert_eq!(snap.no.len(), 1);
    assert_eq!(snap.no[0], (49, 150));
    assert_eq!(snap.yes_dollars.len(), 2);
    assert_eq!(snap.no_dollars.len(), 1);
}

#[test]
fn ws_orderbook_snapshot_deserializes_with_empty_books() {
    let json = r#"{
        "market_ticker": "TEST-MKT",
        "market_id": "xyz789"
    }"#;

    let snap: WsOrderbookSnapshot = serde_json::from_str(json).unwrap();
    assert_eq!(snap.market_ticker, "TEST-MKT");
    assert!(snap.yes.is_empty());
    assert!(snap.no.is_empty());
    assert!(snap.yes_dollars.is_empty());
    assert!(snap.no_dollars.is_empty());
}

#[test]
fn ws_orderbook_delta_deserializes() {
    let json = r#"{
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
    }"#;

    let delta: WsOrderbookDelta = serde_json::from_str(json).unwrap();
    assert_eq!(delta.market_ticker, "INXD-25JAN10-T17900");
    assert_eq!(delta.market_id, "abc123");
    assert_eq!(delta.price, 55);
    assert_eq!(delta.price_dollars, "0.55");
    assert_eq!(delta.delta, 50);
    assert_eq!(delta.delta_fp, "50.00");
    assert_eq!(delta.side, "yes");
    assert_eq!(delta.client_order_id, Some("my-order-1".into()));
    assert_eq!(delta.subaccount, Some(0));
    assert_eq!(delta.ts, Some("2025-01-10T12:00:00Z".into()));
}

#[test]
fn ws_orderbook_delta_deserializes_negative_delta() {
    let json = r#"{
        "market_ticker": "TEST",
        "market_id": "abc",
        "price": 45,
        "price_dollars": "0.45",
        "delta": -100,
        "delta_fp": "-100.00",
        "side": "no"
    }"#;

    let delta: WsOrderbookDelta = serde_json::from_str(json).unwrap();
    assert_eq!(delta.delta, -100);
    assert_eq!(delta.side, "no");
    assert!(delta.client_order_id.is_none());
    assert!(delta.subaccount.is_none());
    assert!(delta.ts.is_none());
}

#[test]
fn ws_fill_deserializes() {
    let json = r#"{
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
    }"#;

    let fill: WsFill = serde_json::from_str(json).unwrap();
    assert_eq!(fill.fill_id, "fill-123");
    assert_eq!(fill.trade_id, "trade-456");
    assert_eq!(fill.order_id, "order-789");
    assert_eq!(fill.client_order_id, Some("my-order".into()));
    assert_eq!(fill.ticker, "INXD-25JAN10-T17900");
    assert_eq!(fill.market_ticker, "INXD-25JAN10-T17900");
    assert_eq!(fill.side, "yes");
    assert_eq!(fill.action, "buy");
    assert_eq!(fill.count, 10);
    assert_eq!(fill.yes_price, 55);
    assert_eq!(fill.no_price, 45);
    assert!(fill.is_taker);
    assert_eq!(fill.fee_cost, "0.05");
    assert_eq!(fill.created_time, Some("2025-01-10T12:00:00Z".into()));
    assert_eq!(fill.subaccount_number, Some(1));
    assert_eq!(fill.ts, Some(1700000000000));
}

#[test]
fn ws_fill_deserializes_without_optional_fields() {
    let json = r#"{
        "fill_id": "fill-123",
        "trade_id": "trade-456",
        "order_id": "order-789",
        "ticker": "TEST",
        "market_ticker": "TEST",
        "side": "no",
        "action": "sell",
        "count": 5,
        "count_fp": "5.00",
        "yes_price": 40,
        "no_price": 60,
        "yes_price_fixed": "0.40",
        "no_price_fixed": "0.60",
        "is_taker": false,
        "fee_cost": "0.00"
    }"#;

    let fill: WsFill = serde_json::from_str(json).unwrap();
    assert_eq!(fill.fill_id, "fill-123");
    assert_eq!(fill.side, "no");
    assert_eq!(fill.action, "sell");
    assert!(!fill.is_taker);
    assert!(fill.client_order_id.is_none());
    assert!(fill.created_time.is_none());
    assert!(fill.subaccount_number.is_none());
    assert!(fill.ts.is_none());
}

#[test]
fn ws_envelope_parse_ticker() {
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
    let ticker = env.parse_ticker().unwrap();
    assert_eq!(ticker.market_ticker, "TEST");
    assert_eq!(ticker.price, 50);
}

#[test]
fn ws_envelope_parse_orderbook_snapshot() {
    let json = r#"{
        "type": "orderbook_snapshot",
        "sid": 2,
        "seq": 1,
        "msg": {
            "market_ticker": "TEST",
            "market_id": "abc",
            "yes": [[50, 100]],
            "no": [[50, 200]]
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let snap = env.parse_orderbook_snapshot().unwrap();
    assert_eq!(snap.market_ticker, "TEST");
    assert_eq!(snap.yes.len(), 1);
    assert_eq!(snap.no.len(), 1);
}

#[test]
fn ws_envelope_parse_orderbook_delta() {
    let json = r#"{
        "type": "orderbook_delta",
        "sid": 2,
        "seq": 5,
        "msg": {
            "market_ticker": "TEST",
            "market_id": "abc",
            "price": 55,
            "price_dollars": "0.55",
            "delta": 25,
            "delta_fp": "25.00",
            "side": "yes"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let delta = env.parse_orderbook_delta().unwrap();
    assert_eq!(delta.market_ticker, "TEST");
    assert_eq!(delta.price, 55);
    assert_eq!(delta.delta, 25);
}

#[test]
fn ws_envelope_parse_fill() {
    let json = r#"{
        "type": "fill",
        "sid": 3,
        "msg": {
            "fill_id": "f1",
            "trade_id": "t1",
            "order_id": "o1",
            "ticker": "TEST",
            "market_ticker": "TEST",
            "side": "yes",
            "action": "buy",
            "count": 10,
            "count_fp": "10.00",
            "yes_price": 50,
            "no_price": 50,
            "yes_price_fixed": "0.50",
            "no_price_fixed": "0.50",
            "is_taker": true,
            "fee_cost": "0.01"
        }
    }"#;

    let env: WsEnvelope = serde_json::from_str(json).unwrap();
    let fill = env.parse_fill().unwrap();
    assert_eq!(fill.fill_id, "f1");
    assert_eq!(fill.action, "buy");
    assert!(fill.is_taker);
}
