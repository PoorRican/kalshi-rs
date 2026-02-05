//! Unit tests for REST type serialization/deserialization.

pub(crate) use cargo_husky as _;
use kalshi_fast::{
    ApplySubaccountTransferResponse, BuySell, CreateOrderRequest, CreateSubaccountResponse,
    ErrorResponse, EventStatus, GetAccountApiLimitsResponse, GetEventsParams,
    GetExchangeAnnouncementsResponse, GetExchangeScheduleResponse, GetExchangeStatusResponse,
    GetFillsParams, GetFillsResponse, GetMarketOrderbookResponse, GetMarketsParams,
    GetOrdersParams, GetPositionsParams, GetSeriesFeeChangesParams, GetSeriesFeeChangesResponse,
    GetSettlementsParams, GetSettlementsResponse, GetSubaccountBalancesResponse,
    GetSubaccountTransfersParams, GetSubaccountTransfersResponse, GetTradesParams,
    GetTradesResponse, GetUserDataTimestampResponse, MarketStatus, MveFilter, OrderStatus,
    OrderType, PositionCountFilter, PriceRange, SelfTradePreventionType, TimeInForce, YesNo,
};

// ============================================================================
// Enum Serialization Tests
// ============================================================================

#[test]
fn market_status_serializes_correctly() {
    assert_eq!(
        serde_json::to_string(&MarketStatus::Open).unwrap(),
        "\"open\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatus::Closed).unwrap(),
        "\"closed\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatus::Settled).unwrap(),
        "\"settled\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatus::Paused).unwrap(),
        "\"paused\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatus::Unopened).unwrap(),
        "\"unopened\""
    );
}

#[test]
fn event_status_serializes_correctly() {
    assert_eq!(
        serde_json::to_string(&EventStatus::Open).unwrap(),
        "\"open\""
    );
    assert_eq!(
        serde_json::to_string(&EventStatus::Closed).unwrap(),
        "\"closed\""
    );
    assert_eq!(
        serde_json::to_string(&EventStatus::Settled).unwrap(),
        "\"settled\""
    );
}

#[test]
fn order_status_serializes_correctly() {
    assert_eq!(
        serde_json::to_string(&OrderStatus::Resting).unwrap(),
        "\"resting\""
    );
    assert_eq!(
        serde_json::to_string(&OrderStatus::Canceled).unwrap(),
        "\"canceled\""
    );
    assert_eq!(
        serde_json::to_string(&OrderStatus::Executed).unwrap(),
        "\"executed\""
    );
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
    assert_eq!(
        serde_json::to_string(&OrderType::Limit).unwrap(),
        "\"limit\""
    );
    assert_eq!(
        serde_json::to_string(&OrderType::Market).unwrap(),
        "\"market\""
    );
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
    assert_eq!(
        serde_json::to_string(&MveFilter::Exclude).unwrap(),
        "\"exclude\""
    );
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

#[test]
fn get_trades_params_serializes_correctly() {
    let params = GetTradesParams {
        ticker: Some("MKT-1".into()),
        min_ts: Some(1000),
        max_ts: Some(2000),
        limit: Some(5),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["ticker"], "MKT-1");
    assert_eq!(json["min_ts"], 1000);
    assert_eq!(json["max_ts"], 2000);
    assert_eq!(json["limit"], 5);
}

#[test]
fn get_fills_params_serializes_correctly() {
    let params = GetFillsParams {
        limit: Some(10),
        ticker: Some("MKT-1".into()),
        subaccount: Some(1),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["limit"], 10);
    assert_eq!(json["ticker"], "MKT-1");
    assert_eq!(json["subaccount"], 1);
}

#[test]
fn get_settlements_params_serializes_correctly() {
    let params = GetSettlementsParams {
        limit: Some(10),
        event_ticker: Some("EVT-1".into()),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["limit"], 10);
    assert_eq!(json["event_ticker"], "EVT-1");
}

// ============================================================================
// Model Deserialization Tests
// ============================================================================

#[test]
fn error_response_deserializes_details_string() {
    let json = r#"{"code":"bad","message":"oops","details":"extra info","service":"svc"}"#;
    let err: ErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(err.code.as_deref(), Some("bad"));
    assert_eq!(err.details.as_deref(), Some("extra info"));
}

#[test]
fn price_range_deserializes_with_aliases() {
    let json = r#"{"min_price":"0.10","max_price":"0.90","increment":"0.05"}"#;
    let range: PriceRange = serde_json::from_str(json).unwrap();
    assert_eq!(range.start, "0.10");
    assert_eq!(range.end, "0.90");
    assert_eq!(range.step, "0.05");
}

#[test]
fn get_series_fee_changes_params_serializes_correctly() {
    let params = GetSeriesFeeChangesParams {
        series_ticker: Some("SERIES-1".into()),
        show_historical: Some(true),
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["series_ticker"], "SERIES-1");
    assert_eq!(json["show_historical"], true);
}

#[test]
fn get_subaccount_transfers_params_serializes_correctly() {
    let params = GetSubaccountTransfersParams {
        cursor: Some("c1".into()),
        limit: Some(20),
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["cursor"], "c1");
    assert_eq!(json["limit"], 20);
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

    let resp: kalshi_fast::GetBalanceResponse = serde_json::from_str(json).unwrap();
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

    let resp: kalshi_fast::GetMarketsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.markets.len(), 2);
    assert_eq!(resp.cursor, Some("next_cursor_token".into()));
}

#[test]
fn get_series_response_deserializes() {
    let json = r#"{
        "series": {"ticker": "SERIES-1", "title": "Example Series"}
    }"#;

    let resp: kalshi_fast::GetSeriesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.series.ticker, "SERIES-1");
    assert_eq!(resp.series.title.as_deref(), Some("Example Series"));
}

#[test]
fn get_markets_response_deserializes_without_cursor() {
    let json = r#"{"markets": []}"#;

    let resp: kalshi_fast::GetMarketsResponse = serde_json::from_str(json).unwrap();
    assert!(resp.markets.is_empty());
    assert!(resp.cursor.is_none());
}

#[test]
fn get_events_response_deserializes() {
    let json = r#"{
        "events": [{"event_ticker": "EVT-1"}],
        "cursor": null
    }"#;

    let resp: kalshi_fast::GetEventsResponse = serde_json::from_str(json).unwrap();
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

    let resp: kalshi_fast::GetPositionsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.market_positions.len(), 1);
    assert!(resp.event_positions.is_empty());
    assert_eq!(resp.cursor, Some("abc123".into()));
}

#[test]
fn positions_page_from_response() {
    let json = r#"{
        "market_positions": [{"ticker": "MKT-1", "position": 100}],
        "event_positions": [{"event_ticker": "EVT-1", "position": 5}],
        "cursor": "abc123"
    }"#;

    let resp: kalshi_fast::GetPositionsResponse = serde_json::from_str(json).unwrap();
    let page: kalshi_fast::PositionsPage = resp.into();
    assert_eq!(page.market_positions.len(), 1);
    assert_eq!(page.event_positions.len(), 1);
}

#[test]
fn get_orders_response_deserializes() {
    let json = r#"{
        "orders": [{"order_id": "ord-1", "ticker": "MKT-1", "status": "resting"}]
    }"#;

    let resp: kalshi_fast::GetOrdersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.orders.len(), 1);
    assert!(resp.cursor.is_none());
}

#[test]
fn create_order_response_deserializes() {
    let json = r#"{
        "order": {"order_id": "ord-123", "ticker": "MKT-1", "status": "resting"}
    }"#;

    let resp: kalshi_fast::CreateOrderResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.order.order_id, "ord-123");
    assert_eq!(resp.order.ticker, "MKT-1");
}

#[test]
fn cancel_order_response_deserializes() {
    let json = r#"{
        "order": {"order_id": "ord-123", "ticker": "MKT-1", "status": "canceled"},
        "reduced_by": 5,
        "reduced_by_fp": "5.0"
    }"#;

    let resp: kalshi_fast::CancelOrderResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.order.status, Some(OrderStatus::Canceled));
    assert_eq!(resp.reduced_by, 5);
    assert_eq!(resp.reduced_by_fp, "5.0");
}

#[test]
fn get_market_orderbook_response_deserializes() {
    let json = r#"{
        "orderbook": {
            "yes": [[50, 100]],
            "no": [[49, 200]],
            "yes_dollars": [["0.50", 100]],
            "no_dollars": [["0.49", 200]]
        },
        "orderbook_fp": {
            "yes_dollars": [["0.50", "100.00"]],
            "no_dollars": [["0.49", "200.00"]]
        }
    }"#;

    let resp: GetMarketOrderbookResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.orderbook.yes.len(), 1);
    assert!(resp.orderbook_fp.is_some());
}

#[test]
fn get_trades_response_deserializes() {
    let json = r#"{
        "trades": [{"trade_id":"t1","ticker":"MKT-1","price":55}],
        "cursor": "c1"
    }"#;

    let resp: GetTradesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.trades.len(), 1);
    assert_eq!(resp.cursor, Some("c1".into()));
}

#[test]
fn get_exchange_status_response_deserializes() {
    let json = r#"{
        "exchange_active": true,
        "trading_active": false,
        "exchange_estimated_resume_time": "2025-01-01T00:00:00Z"
    }"#;

    let resp: GetExchangeStatusResponse = serde_json::from_str(json).unwrap();
    assert!(resp.exchange_active);
    assert!(!resp.trading_active);
    assert_eq!(
        resp.exchange_estimated_resume_time.as_deref(),
        Some("2025-01-01T00:00:00Z")
    );
}

#[test]
fn get_exchange_announcements_response_deserializes() {
    let json = r#"{
        "announcements": [
            {"type":"info","message":"hello","delivery_time":"2025-01-01T00:00:00Z","status":"active"}
        ]
    }"#;

    let resp: GetExchangeAnnouncementsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.announcements.len(), 1);
    assert_eq!(resp.announcements[0].message, "hello");
}

#[test]
fn get_exchange_schedule_response_deserializes() {
    let json = r#"{
        "schedule": {
            "standard_hours": [
                {
                    "start_time":"09:00",
                    "end_time":"17:00",
                    "monday":[{"open_time":"09:00","close_time":"17:00"}]
                }
            ],
            "maintenance_windows": [
                {"start_datetime":"2025-01-01T00:00:00Z","end_datetime":"2025-01-01T01:00:00Z"}
            ]
        }
    }"#;

    let resp: GetExchangeScheduleResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.schedule.standard_hours.len(), 1);
    assert_eq!(resp.schedule.maintenance_windows.len(), 1);
}

#[test]
fn get_user_data_timestamp_response_deserializes() {
    let json = r#"{"as_of_time":"2025-01-01T00:00:00Z"}"#;

    let resp: GetUserDataTimestampResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.as_of_time, "2025-01-01T00:00:00Z");
}

#[test]
fn get_series_fee_changes_response_deserializes() {
    let json = r#"{
        "series_fee_change_arr": [
            {"id":1,"series_ticker":"SERIES-1","fee_type":"flat","fee_multiplier":5,"scheduled_ts":1700000000}
        ]
    }"#;

    let resp: GetSeriesFeeChangesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.series_fee_change_arr.len(), 1);
    assert_eq!(resp.series_fee_change_arr[0].series_ticker, "SERIES-1");
}

#[test]
fn get_fills_response_deserializes() {
    let json = r#"{
        "fills": [{"fill_id":"f1","order_id":"o1","trade_id":"t1","ticker":"MKT-1"}],
        "cursor": "c1"
    }"#;

    let resp: GetFillsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.fills.len(), 1);
    assert_eq!(resp.cursor, Some("c1".into()));
}

#[test]
fn get_settlements_response_deserializes() {
    let json = r#"{
        "settlements": [{"settlement_id":"s1","ticker":"MKT-1"}],
        "cursor": null
    }"#;

    let resp: GetSettlementsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.settlements.len(), 1);
    assert!(resp.cursor.is_none());
}

#[test]
fn get_account_api_limits_response_deserializes() {
    let json = r#"{"usage_tier":"basic","read_limit":20,"write_limit":10}"#;

    let resp: GetAccountApiLimitsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.usage_tier, "basic");
    assert_eq!(resp.read_limit, 20);
    assert_eq!(resp.write_limit, 10);
}

#[test]
fn get_subaccount_balances_response_deserializes() {
    let json = r#"{
        "subaccount_balances": [{"subaccount_number":1,"balance":100,"updated_ts":1700000000}]
    }"#;

    let resp: GetSubaccountBalancesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.subaccount_balances.len(), 1);
    assert_eq!(resp.subaccount_balances[0].balance, "100");
}

#[test]
fn create_subaccount_response_deserializes() {
    let json = r#"{"subaccount_number": 2}"#;

    let resp: CreateSubaccountResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.subaccount_number, 2);
}

#[test]
fn get_subaccount_transfers_response_deserializes() {
    let json = r#"{
        "subaccount_transfers": [
            {"transfer_id":"t1","from_subaccount":0,"to_subaccount":1,"amount_cents":100,"created_ts":1700000000}
        ],
        "cursor": "c1"
    }"#;

    let resp: GetSubaccountTransfersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.subaccount_transfers.len(), 1);
    assert_eq!(resp.cursor, Some("c1".into()));
}

#[test]
fn apply_subaccount_transfer_response_deserializes() {
    let json = r#"{}"#;
    let _resp: ApplySubaccountTransferResponse = serde_json::from_str(json).unwrap();
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
        event_ticker: Some(
            vec!["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        ..Default::default()
    };
    assert!(params.validate().is_ok());

    // 11 is too many
    let params = GetMarketsParams {
        event_ticker: Some(
            vec![
                "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10", "E11",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
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
        event_ticker: Some(
            vec![
                "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "E10", "E11",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
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

#[test]
fn create_order_request_validate_requires_count_or_count_fp() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_rejects_count_mismatch() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(2),
        count_fp: Some("1.0".into()),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_rejects_conflicting_prices() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        yes_price: Some(10),
        yes_price_dollars: Some("0.10".into()),
        ..Default::default()
    };
    assert!(req.validate().is_err());

    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        yes_price: Some(10),
        no_price: Some(90),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_market_order_no_price() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        r#type: Some(OrderType::Market),
        yes_price: Some(10),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_limit_order_requires_price() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        r#type: Some(OrderType::Limit),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_subaccount_bounds() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        yes_price: Some(10),
        subaccount: Some(32),
        ..Default::default()
    };
    assert!(req.validate().is_ok());

    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        yes_price: Some(10),
        subaccount: Some(33),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_sell_position_floor() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        yes_price: Some(10),
        sell_position_floor: Some(1),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn create_order_request_validate_ok_with_yes_price() {
    let req = CreateOrderRequest {
        ticker: "TICK-1".into(),
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        yes_price: Some(10),
        r#type: Some(OrderType::Limit),
        ..Default::default()
    };
    assert!(req.validate().is_ok());
}
