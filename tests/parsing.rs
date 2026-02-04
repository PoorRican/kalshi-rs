//! Unit tests for REST type serialization/deserialization.

use kalshi::{
    BuySell, CreateOrderRequest, EventStatus, GetEventsParams, GetMarketsParams,
    GetOrdersParams, GetPositionsParams, MarketStatus, MveFilter, OrderStatus, OrderType,
    PositionCountFilter, SelfTradePreventionType, TimeInForce, YesNo,
};

// ============================================================================
// Enum Serialization Tests
// ============================================================================

#[test]
fn market_status_serializes_correctly() {
    assert_eq!(serde_json::to_string(&MarketStatus::Open).unwrap(), "\"open\"");
    assert_eq!(serde_json::to_string(&MarketStatus::Closed).unwrap(), "\"closed\"");
    assert_eq!(serde_json::to_string(&MarketStatus::Settled).unwrap(), "\"settled\"");
    assert_eq!(serde_json::to_string(&MarketStatus::Paused).unwrap(), "\"paused\"");
    assert_eq!(serde_json::to_string(&MarketStatus::Unopened).unwrap(), "\"unopened\"");
}

#[test]
fn event_status_serializes_correctly() {
    assert_eq!(serde_json::to_string(&EventStatus::Open).unwrap(), "\"open\"");
    assert_eq!(serde_json::to_string(&EventStatus::Closed).unwrap(), "\"closed\"");
    assert_eq!(serde_json::to_string(&EventStatus::Settled).unwrap(), "\"settled\"");
}

#[test]
fn order_status_serializes_correctly() {
    assert_eq!(serde_json::to_string(&OrderStatus::Resting).unwrap(), "\"resting\"");
    assert_eq!(serde_json::to_string(&OrderStatus::Canceled).unwrap(), "\"canceled\"");
    assert_eq!(serde_json::to_string(&OrderStatus::Executed).unwrap(), "\"executed\"");
}

#[test]
fn yes_no_serializes_correctly() {
    assert_eq!(serde_json::to_string(&YesNo::Yes).unwrap(), "\"yes\"");
    assert_eq!(serde_json::to_string(&YesNo::No).unwrap(), "\"no\"");
}

#[test]
fn buy_sell_serializes_correctly() {
    assert_eq!(serde_json::to_string(&BuySell::Buy).unwrap(), "\"buy\"");
    assert_eq!(serde_json::to_string(&BuySell::Sell).unwrap(), "\"sell\"");
}

#[test]
fn order_type_serializes_correctly() {
    assert_eq!(serde_json::to_string(&OrderType::Limit).unwrap(), "\"limit\"");
    assert_eq!(serde_json::to_string(&OrderType::Market).unwrap(), "\"market\"");
}

#[test]
fn time_in_force_serializes_correctly() {
    assert_eq!(
        serde_json::to_string(&TimeInForce::FillOrKill).unwrap(),
        "\"fill_or_kill\""
    );
    assert_eq!(
        serde_json::to_string(&TimeInForce::GoodTillCanceled).unwrap(),
        "\"good_till_canceled\""
    );
    assert_eq!(
        serde_json::to_string(&TimeInForce::ImmediateOrCancel).unwrap(),
        "\"immediate_or_cancel\""
    );
}

#[test]
fn mve_filter_serializes_correctly() {
    assert_eq!(serde_json::to_string(&MveFilter::Only).unwrap(), "\"only\"");
    assert_eq!(serde_json::to_string(&MveFilter::Exclude).unwrap(), "\"exclude\"");
}

#[test]
fn self_trade_prevention_type_serializes_correctly() {
    assert_eq!(
        serde_json::to_string(&SelfTradePreventionType::TakerAtCross).unwrap(),
        "\"taker_at_cross\""
    );
    assert_eq!(
        serde_json::to_string(&SelfTradePreventionType::Maker).unwrap(),
        "\"maker\""
    );
}

// ============================================================================
// Request Params Serialization Tests
// ============================================================================

#[test]
fn get_markets_params_serializes_with_csv_fields() {
    let params = GetMarketsParams {
        limit: Some(50),
        status: Some(MarketStatus::Open),
        event_ticker: Some(vec!["EVT1".into(), "EVT2".into()]),
        tickers: Some(vec!["TKR1".into(), "TKR2".into(), "TKR3".into()]),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["limit"], 50);
    assert_eq!(json["status"], "open");
    assert_eq!(json["event_ticker"], "EVT1,EVT2");
    assert_eq!(json["tickers"], "TKR1,TKR2,TKR3");
}

#[test]
fn get_markets_params_omits_none_fields() {
    let params = GetMarketsParams {
        limit: Some(100),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["limit"], 100);
    assert!(json.get("cursor").is_none());
    assert!(json.get("event_ticker").is_none());
    assert!(json.get("status").is_none());
}

#[test]
fn get_events_params_serializes_correctly() {
    let params = GetEventsParams {
        limit: Some(100),
        status: Some(EventStatus::Open),
        with_nested_markets: Some(true),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["limit"], 100);
    assert_eq!(json["status"], "open");
    assert_eq!(json["with_nested_markets"], true);
}

#[test]
fn get_positions_params_serializes_count_filter_csv() {
    let params = GetPositionsParams {
        count_filter: Some(vec![
            PositionCountFilter::Position,
            PositionCountFilter::TotalTraded,
        ]),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["count_filter"], "position,total_traded");
}

#[test]
fn create_order_request_serializes_all_fields() {
    let req = CreateOrderRequest {
        ticker: "TICK-123".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        client_order_id: Some("my-order-1".into()),
        count: Some(10),
        r#type: Some(OrderType::Limit),
        yes_price: Some(50),
        time_in_force: Some(TimeInForce::GoodTillCanceled),
        post_only: Some(true),
        subaccount: Some(1),
        ..Default::default()
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["ticker"], "TICK-123");
    assert_eq!(json["side"], "yes");
    assert_eq!(json["action"], "buy");
    assert_eq!(json["client_order_id"], "my-order-1");
    assert_eq!(json["count"], 10);
    assert_eq!(json["type"], "limit");
    assert_eq!(json["yes_price"], 50);
    assert_eq!(json["time_in_force"], "good_till_canceled");
    assert_eq!(json["post_only"], true);
    assert_eq!(json["subaccount"], 1);
}

// ============================================================================
// Response Deserialization Tests
// ============================================================================

#[test]
fn get_balance_response_deserializes() {
    let json = r#"{
        "balance": 100000,
        "portfolio_value": 50000,
        "updated_ts": 1700000000
    }"#;

    let resp: kalshi::GetBalanceResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.balance, 100000);
    assert_eq!(resp.portfolio_value, 50000);
    assert_eq!(resp.updated_ts, 1700000000);
}

#[test]
fn get_markets_response_deserializes() {
    let json = r#"{
        "markets": [{"ticker": "MKT-1"}, {"ticker": "MKT-2"}],
        "cursor": "next_cursor_token"
    }"#;

    let resp: kalshi::GetMarketsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.markets.len(), 2);
    assert_eq!(resp.cursor, Some("next_cursor_token".into()));
}

#[test]
fn get_markets_response_deserializes_without_cursor() {
    let json = r#"{"markets": []}"#;

    let resp: kalshi::GetMarketsResponse = serde_json::from_str(json).unwrap();
    assert!(resp.markets.is_empty());
    assert!(resp.cursor.is_none());
}

#[test]
fn get_events_response_deserializes() {
    let json = r#"{
        "events": [{"event_ticker": "EVT-1"}],
        "cursor": null
    }"#;

    let resp: kalshi::GetEventsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.events.len(), 1);
    assert!(resp.cursor.is_none());
}

#[test]
fn get_positions_response_deserializes() {
    let json = r#"{
        "market_positions": [{"ticker": "MKT-1", "position": 100}],
        "event_positions": [],
        "cursor": "abc123"
    }"#;

    let resp: kalshi::GetPositionsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.market_positions.len(), 1);
    assert!(resp.event_positions.is_empty());
    assert_eq!(resp.cursor, Some("abc123".into()));
}

#[test]
fn get_orders_response_deserializes() {
    let json = r#"{
        "orders": [{"order_id": "ord-1", "status": "resting"}]
    }"#;

    let resp: kalshi::GetOrdersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.orders.len(), 1);
    assert!(resp.cursor.is_none());
}

#[test]
fn create_order_response_deserializes() {
    let json = r#"{
        "order": {"order_id": "ord-123", "ticker": "MKT-1", "status": "resting"}
    }"#;

    let resp: kalshi::CreateOrderResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.order["order_id"], "ord-123");
}

#[test]
fn cancel_order_response_deserializes() {
    let json = r#"{
        "order": {"order_id": "ord-123", "status": "canceled"},
        "reduced_by": 5,
        "reduced_by_fp": "5.0"
    }"#;

    let resp: kalshi::CancelOrderResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.order["status"], "canceled");
    assert_eq!(resp.reduced_by, 5);
    assert_eq!(resp.reduced_by_fp, "5.0");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn get_markets_params_validates_limit_bounds() {
    // Zero is invalid
    let params = GetMarketsParams {
        limit: Some(0),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // Over 1000 is invalid
    let params = GetMarketsParams {
        limit: Some(1001),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // 1000 is valid
    let params = GetMarketsParams {
        limit: Some(1000),
        ..Default::default()
    };
    assert!(params.validate().is_ok());

    // 1 is valid
    let params = GetMarketsParams {
        limit: Some(1),
        ..Default::default()
    };
    assert!(params.validate().is_ok());
}

#[test]
fn get_markets_params_validates_event_ticker_count() {
    // 10 is ok
    let params = GetMarketsParams {
        event_ticker: Some(vec!["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10"]
            .into_iter().map(String::from).collect()),
        ..Default::default()
    };
    assert!(params.validate().is_ok());

    // 11 is too many
    let params = GetMarketsParams {
        event_ticker: Some(vec!["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10", "E11"]
            .into_iter().map(String::from).collect()),
        ..Default::default()
    };
    assert!(params.validate().is_err());
}

#[test]
fn get_markets_params_validates_timestamp_mutual_exclusion() {
    // created_ts and close_ts together is invalid
    let params = GetMarketsParams {
        min_created_ts: Some(1000),
        min_close_ts: Some(2000),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // created_ts and settled_ts together is invalid
    let params = GetMarketsParams {
        max_created_ts: Some(1000),
        min_settled_ts: Some(2000),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // min_updated_ts cannot combine with other filters
    let params = GetMarketsParams {
        min_updated_ts: Some(1000),
        status: Some(MarketStatus::Open),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // min_updated_ts with mve_filter=only is invalid
    let params = GetMarketsParams {
        min_updated_ts: Some(1000),
        mve_filter: Some(MveFilter::Only),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    // min_updated_ts with mve_filter=exclude is valid
    let params = GetMarketsParams {
        min_updated_ts: Some(1000),
        mve_filter: Some(MveFilter::Exclude),
        ..Default::default()
    };
    assert!(params.validate().is_ok());
}

#[test]
fn get_events_params_validates_limit_bounds() {
    let params = GetEventsParams {
        limit: Some(0),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    let params = GetEventsParams {
        limit: Some(201),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    let params = GetEventsParams {
        limit: Some(200),
        ..Default::default()
    };
    assert!(params.validate().is_ok());
}

#[test]
fn get_positions_params_validates_subaccount_bounds() {
    let params = GetPositionsParams {
        subaccount: Some(32),
        ..Default::default()
    };
    assert!(params.validate().is_ok());

    let params = GetPositionsParams {
        subaccount: Some(33),
        ..Default::default()
    };
    assert!(params.validate().is_err());
}

#[test]
fn get_orders_params_validates_limit_bounds() {
    let params = GetOrdersParams {
        limit: Some(0),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    let params = GetOrdersParams {
        limit: Some(201),
        ..Default::default()
    };
    assert!(params.validate().is_err());

    let params = GetOrdersParams {
        limit: Some(200),
        ..Default::default()
    };
    assert!(params.validate().is_ok());
}

#[test]
fn get_orders_params_validates_event_ticker_count() {
    let params = GetOrdersParams {
        event_ticker: Some(vec!["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10", "E11"]
            .into_iter().map(String::from).collect()),
        ..Default::default()
    };
    assert!(params.validate().is_err());
}

#[test]
fn get_orders_params_validates_subaccount_bounds() {
    let params = GetOrdersParams {
        subaccount: Some(33),
        ..Default::default()
    };
    assert!(params.validate().is_err());
}
