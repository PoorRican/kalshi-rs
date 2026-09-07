//! Unit tests for REST type serialization/deserialization.

pub(crate) use cargo_husky as _;
use kalshi_fast::{
    ApplySubaccountTransferResponse, BookSide, BuySell, CreateOrderRequest,
    CreateSubaccountResponse, ErrorResponse, EventData, EventMetadata, EventStatus,
    GetAccountApiLimitsResponse, GetAccountEndpointCostsResponse, GetEventsParams,
    GetExchangeScheduleResponse, GetExchangeStatusResponse, GetFillsParams, GetFillsResponse,
    GetMarketOrderbookResponse, GetMarketsParams, GetOrderQueuePositionsParams, GetOrdersParams,
    GetPositionsParams, GetSeriesFeeChangesParams, GetSeriesFeeChangesResponse,
    GetSettlementsParams, GetSettlementsResponse, GetSubaccountBalancesResponse,
    GetSubaccountTransfersParams, GetSubaccountTransfersResponse, GetTradesParams,
    GetTradesResponse, GetUserDataTimestampResponse, MarketMetadata, MarketStatus,
    MarketStatusConversionError, MarketStatusQuery, MveFilter, OrderStatus, OrderType,
    PositionCountFilter, PriceRange, SelfTradePreventionType, TimeInForce, TradeTakerSide, YesNo,
};

// ============================================================================
// Enum Serialization Tests
// ============================================================================

#[test]
fn market_status_query_serializes_correctly() {
    assert_eq!(
        serde_json::to_string(&MarketStatusQuery::Open).unwrap(),
        "\"open\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatusQuery::Closed).unwrap(),
        "\"closed\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatusQuery::Settled).unwrap(),
        "\"settled\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatusQuery::Paused).unwrap(),
        "\"paused\""
    );
    assert_eq!(
        serde_json::to_string(&MarketStatusQuery::Unopened).unwrap(),
        "\"unopened\""
    );
}

#[test]
fn market_status_from_lifecycle_is_best_effort() {
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Initialized),
        MarketStatusQuery::Unopened
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Inactive),
        MarketStatusQuery::Paused
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Active),
        MarketStatusQuery::Open
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Closed),
        MarketStatusQuery::Closed
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Determined),
        MarketStatusQuery::Closed
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Disputed),
        MarketStatusQuery::Closed
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Amended),
        MarketStatusQuery::Closed
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Finalized),
        MarketStatusQuery::Settled
    );
    assert_eq!(
        MarketStatusQuery::from(MarketStatus::Unknown),
        MarketStatusQuery::Unknown
    );
}

#[test]
fn market_status_from_query_is_best_effort() {
    assert_eq!(
        MarketStatus::from(MarketStatusQuery::Unopened),
        MarketStatus::Initialized
    );
    assert_eq!(
        MarketStatus::from(MarketStatusQuery::Open),
        MarketStatus::Active
    );
    assert_eq!(
        MarketStatus::from(MarketStatusQuery::Paused),
        MarketStatus::Inactive
    );
    assert_eq!(
        MarketStatus::from(MarketStatusQuery::Closed),
        MarketStatus::Closed
    );
    assert_eq!(
        MarketStatus::from(MarketStatusQuery::Settled),
        MarketStatus::Finalized
    );
    assert_eq!(
        MarketStatus::from(MarketStatusQuery::Unknown),
        MarketStatus::Unknown
    );
}

#[test]
fn market_status_query_try_from_lifecycle_is_strict() {
    assert_eq!(
        MarketStatusQuery::try_from(&MarketStatus::Closed).unwrap(),
        MarketStatusQuery::Closed
    );
    assert_eq!(
        MarketStatusQuery::try_from(&MarketStatus::Unknown).unwrap(),
        MarketStatusQuery::Unknown
    );

    assert!(matches!(
        MarketStatusQuery::try_from(&MarketStatus::Active),
        Err(MarketStatusConversionError::LifecycleToQuery(
            MarketStatus::Active
        ))
    ));
}

#[test]
fn market_status_try_from_query_is_strict() {
    assert_eq!(
        MarketStatus::try_from(&MarketStatusQuery::Closed).unwrap(),
        MarketStatus::Closed
    );
    assert_eq!(
        MarketStatus::try_from(&MarketStatusQuery::Unknown).unwrap(),
        MarketStatus::Unknown
    );

    assert!(matches!(
        MarketStatus::try_from(&MarketStatusQuery::Open),
        Err(MarketStatusConversionError::QueryToLifecycle(
            MarketStatusQuery::Open
        ))
    ));
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
        status: Some(MarketStatusQuery::Open),
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
    let json = r#"{"code":"bad","message":"oops","details":"extra info"}"#;
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

#[test]
fn historical_params_serialize_correctly() {
    let markets = kalshi_fast::GetHistoricalMarketsParams {
        limit: Some(100),
        cursor: Some("c2".into()),
        tickers: Some("MKT-1,MKT-2".into()),
        series_ticker: None,
        event_ticker: Some("EVT-1".into()),
        mve_filter: Some(MveFilter::Exclude),
    };
    let markets_json = serde_json::to_value(&markets).unwrap();
    assert_eq!(markets_json["limit"], 100);
    assert_eq!(markets_json["cursor"], "c2");
    assert_eq!(markets_json["tickers"], "MKT-1,MKT-2");
    assert_eq!(markets_json["event_ticker"], "EVT-1");
    assert_eq!(markets_json["mve_filter"], "exclude");

    let fills = kalshi_fast::GetHistoricalFillsParams {
        ticker: Some("MKT-1".into()),
        max_ts: Some(1700000000),
        limit: Some(25),
        cursor: Some("c3".into()),
    };
    let fills_json = serde_json::to_value(&fills).unwrap();
    assert_eq!(fills_json["ticker"], "MKT-1");
    assert_eq!(fills_json["max_ts"], 1700000000);
    assert_eq!(fills_json["limit"], 25);
    assert_eq!(fills_json["cursor"], "c3");

    let orders = kalshi_fast::GetHistoricalOrdersParams {
        ticker: Some("MKT-1".into()),
        max_ts: Some(1700000100),
        limit: Some(15),
        cursor: Some("c4".into()),
    };
    let orders_json = serde_json::to_value(&orders).unwrap();
    assert_eq!(orders_json["ticker"], "MKT-1");
    assert_eq!(orders_json["max_ts"], 1700000100);
    assert_eq!(orders_json["limit"], 15);
    assert_eq!(orders_json["cursor"], "c4");

    let candlesticks = kalshi_fast::GetMarketCandlesticksHistoricalParams {
        start_ts: 1700000000,
        end_ts: 1700003600,
        period_interval: 60,
    };
    let candlesticks_json = serde_json::to_value(&candlesticks).unwrap();
    assert_eq!(candlesticks_json["start_ts"], 1700000000);
    assert_eq!(candlesticks_json["end_ts"], 1700003600);
    assert_eq!(candlesticks_json["period_interval"], 60);
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
        "markets": [
            {
                "ticker": "MKT-1",
                "event_ticker": "EVT-1",
                "series_ticker": "SERIES-1",
                "status": "active",
                "volume_fp": "125.50",
                "volume_24h_fp": "101.25",
                "open_interest_fp": "500.00",
                "occurrence_datetime": "2026-04-16T18:30:00Z"
            },
            {"ticker": "MKT-2"}
        ],
        "cursor": "next_cursor_token"
    }"#;

    let resp: kalshi_fast::GetMarketsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.markets.len(), 2);
    assert_eq!(resp.markets[0].event_ticker.as_deref(), Some("EVT-1"));
    assert_eq!(resp.markets[0].series_ticker.as_deref(), Some("SERIES-1"));
    assert_eq!(
        resp.markets[0].status,
        Some(kalshi_fast::MarketStatus::Active)
    );
    assert_eq!(resp.markets[0].volume_fp.as_deref(), Some("125.50"));
    assert_eq!(resp.markets[0].volume_24h_fp.as_deref(), Some("101.25"));
    assert_eq!(resp.markets[0].open_interest_fp.as_deref(), Some("500.00"));
    assert_eq!(
        resp.markets[0].occurrence_datetime.as_deref(),
        Some("2026-04-16T18:30:00Z")
    );
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
    assert!(resp.milestones.is_empty());
    assert!(resp.cursor.is_none());
}

#[test]
fn get_events_response_deserializes_with_milestones() {
    let json = r#"{
        "events": [{"event_ticker": "EVT-1"}],
        "cursor": "next",
        "milestones": [{
            "id": "ms-1",
            "category": "politics",
            "type": "debate",
            "start_date": "2023-11-07T05:31:56Z",
            "related_event_tickers": ["EVT-1"],
            "title": "Debate Night",
            "notification_message": "Debate starts now",
            "details": {"network": "PBS"},
            "primary_event_tickers": ["EVT-1"],
            "last_updated_ts": "2023-11-07T05:31:56Z",
            "end_date": "2023-11-08T05:31:56Z",
            "source_id": "src-1",
            "source_ids": {"provider": "kalshi"}
        }]
    }"#;

    let resp: kalshi_fast::GetEventsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.events.len(), 1);
    assert_eq!(resp.cursor.as_deref(), Some("next"));
    assert_eq!(resp.milestones.len(), 1);
    assert_eq!(resp.milestones[0].id.as_deref(), Some("ms-1"));
    assert_eq!(resp.milestones[0].milestone_type.as_deref(), Some("debate"));
}

#[test]
fn get_event_response_deserializes_rich_schema_fields() {
    let json = r#"{
        "event": {
            "event_ticker": "EVT-1",
            "series_ticker": "SER-1",
            "sub_title": "Sub",
            "title": "Title",
            "collateral_return_type": "binary",
            "mutually_exclusive": true,
            "category": "Politics",
            "product_metadata": {"cadence": "fifteen_min"},
            "settlement_sources": [{"name": "NWS", "url": "https://weather.gov"}],
            "exchange_index": 0,
            "strike_date": "2023-11-07T05:31:56Z",
            "strike_period": "day",
            "last_updated_ts": "2023-11-07T05:31:56Z",
            "markets": []
        },
        "markets": [{
            "ticker": "MKT-1",
            "event_ticker": "EVT-1",
            "market_type": "binary",
            "yes_bid_size_fp": "10.00",
            "yes_ask_size_fp": "11.00",
            "settlement_timer_seconds": 123,
            "notional_value": 100,
            "notional_value_dollars": "1.0000",
            "previous_yes_bid": 50,
            "previous_yes_bid_dollars": "0.5000",
            "previous_yes_ask": 55,
            "previous_yes_ask_dollars": "0.5500",
            "previous_price": 52,
            "previous_price_dollars": "0.5200",
            "liquidity": 1000,
            "liquidity_dollars": "10.0000",
            "expiration_value": "123.4",
            "occurrence_datetime": "2026-04-16T18:30:00Z",
            "tick_size": 1,
            "expected_expiration_time": "2023-11-07T05:31:56Z",
            "settlement_value": 100,
            "settlement_value_dollars": "1.0000",
            "settlement_ts": "2023-11-07T05:31:56Z",
            "fee_waiver_expiration_time": "2023-11-07T05:31:56Z",
            "early_close_condition": "close when event occurs",
            "strike_type": "greater",
            "floor_strike": 100.0,
            "cap_strike": 200.0,
            "functional_strike": "f(x)",
            "mve_collection_ticker": "COL-1",
            "primary_participant_key": "pk-1",
            "is_provisional": true,
            "price_ranges": [{"start": "0.0000", "end": "1.0000", "step": "0.0100"}]
        }]
    }"#;

    let resp: kalshi_fast::GetEventResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.event.event_ticker, "EVT-1");
    assert_eq!(resp.event.collateral_return_type.as_deref(), Some("binary"));
    assert_eq!(resp.event.mutually_exclusive, Some(true));
    assert_eq!(resp.event.settlement_sources.len(), 1);
    assert_eq!(
        resp.event.settlement_sources[0].name.as_deref(),
        Some("NWS")
    );
    assert_eq!(resp.event.exchange_index, Some(0));
    assert_eq!(
        resp.event
            .product_metadata
            .as_ref()
            .and_then(|m| m.cadence.as_deref()),
        Some("fifteen_min")
    );
    assert_eq!(resp.markets.len(), 1);
    assert_eq!(resp.markets[0].yes_bid_size_fp.as_deref(), Some("10.00"));
    assert_eq!(
        resp.markets[0].liquidity_dollars.as_deref(),
        Some("10.0000")
    );
    assert_eq!(
        resp.markets[0].occurrence_datetime.as_deref(),
        Some("2026-04-16T18:30:00Z")
    );
    assert_eq!(resp.markets[0].strike_type.as_deref(), Some("greater"));
}

#[test]
fn batch_get_market_candlesticks_response_deserializes_typed() {
    let json = r#"{
        "markets": [{
            "market_ticker": "MKT-1",
            "candlesticks": [{
                "end_period_ts": 123,
                "yes_bid": {
                    "open": 10, "open_dollars": "0.1000",
                    "low": 9, "low_dollars": "0.0900",
                    "high": 11, "high_dollars": "0.1100",
                    "close": 10, "close_dollars": "0.1000"
                },
                "yes_ask": {
                    "open": 20, "open_dollars": "0.2000",
                    "low": 19, "low_dollars": "0.1900",
                    "high": 21, "high_dollars": "0.2100",
                    "close": 20, "close_dollars": "0.2000"
                },
                "price": {
                    "open": 15, "open_dollars": "0.1500",
                    "low": 14, "low_dollars": "0.1400",
                    "high": 16, "high_dollars": "0.1600",
                    "close": 15, "close_dollars": "0.1500",
                    "mean": 15, "mean_dollars": "0.1500",
                    "previous": 13, "previous_dollars": "0.1300",
                    "min": 12, "min_dollars": "0.1200",
                    "max": 17, "max_dollars": "0.1700"
                },
                "volume": 123,
                "volume_fp": "10.00",
                "open_interest": 456,
                "open_interest_fp": "20.00"
            }]
        }]
    }"#;

    let resp: kalshi_fast::BatchGetMarketCandlesticksResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.markets.len(), 1);
    assert_eq!(resp.markets[0].market_ticker, "MKT-1");
    assert_eq!(resp.markets[0].candlesticks.len(), 1);
    assert_eq!(resp.markets[0].candlesticks[0].price.mean, Some(15));
}

#[test]
fn get_event_candlesticks_response_deserializes_nested_market_arrays() {
    let json = r#"{
        "market_tickers": ["MKT-1"],
        "market_candlesticks": [[{
            "end_period_ts": 123,
            "yes_bid": {
                "open": 10, "open_dollars": "0.1000",
                "low": 9, "low_dollars": "0.0900",
                "high": 11, "high_dollars": "0.1100",
                "close": 10, "close_dollars": "0.1000"
            },
            "yes_ask": {
                "open": 20, "open_dollars": "0.2000",
                "low": 19, "low_dollars": "0.1900",
                "high": 21, "high_dollars": "0.2100",
                "close": 20, "close_dollars": "0.2000"
            },
            "price": {
                "open": 15, "open_dollars": "0.1500",
                "low": 14, "low_dollars": "0.1400",
                "high": 16, "high_dollars": "0.1600",
                "close": 15, "close_dollars": "0.1500",
                "mean": 15, "mean_dollars": "0.1500",
                "previous": 13, "previous_dollars": "0.1300",
                "min": 12, "min_dollars": "0.1200",
                "max": 17, "max_dollars": "0.1700"
            },
            "volume": 123,
            "volume_fp": "10.00",
            "open_interest": 456,
            "open_interest_fp": "20.00"
        }]],
        "adjusted_end_ts": 123
    }"#;

    let resp: kalshi_fast::GetEventCandlesticksResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.market_tickers, vec!["MKT-1"]);
    assert_eq!(resp.market_candlesticks.len(), 1);
    assert_eq!(resp.market_candlesticks[0].len(), 1);
    assert_eq!(resp.market_candlesticks[0][0].open_interest_fp, "20.00");
}

#[test]
fn batch_get_market_candlesticks_response_deserializes_synthetic_null_ohlc() {
    let json = r#"{
        "markets": [{
            "market_ticker": "MKT-1",
            "candlesticks": [{
                "end_period_ts": 124,
                "yes_bid": {
                    "open": null, "open_dollars": null,
                    "low": null, "low_dollars": null,
                    "high": null, "high_dollars": null,
                    "close": null, "close_dollars": null
                },
                "yes_ask": {
                    "open": null, "open_dollars": null,
                    "low": null, "low_dollars": null,
                    "high": null, "high_dollars": null,
                    "close": null, "close_dollars": null
                },
                "price": {
                    "open": null, "open_dollars": null,
                    "low": null, "low_dollars": null,
                    "high": null, "high_dollars": null,
                    "close": null, "close_dollars": null,
                    "mean": null, "mean_dollars": null,
                    "previous": null, "previous_dollars": null,
                    "min": null, "min_dollars": null,
                    "max": null, "max_dollars": null
                },
                "volume": 0,
                "volume_fp": "0.00",
                "open_interest": 0,
                "open_interest_fp": "0.00"
            }]
        }]
    }"#;

    let resp: kalshi_fast::BatchGetMarketCandlesticksResponse = serde_json::from_str(json).unwrap();
    let candle = &resp.markets[0].candlesticks[0];
    assert_eq!(candle.yes_bid.open, None);
    assert_eq!(candle.yes_ask.close_dollars, None);
    assert_eq!(candle.price.open, None);
    assert_eq!(candle.price.mean_dollars, None);
}

#[test]
fn get_market_candlesticks_historical_response_deserializes() {
    let json = r#"{
        "ticker": "MKT-1",
        "candlesticks": [{
            "end_period_ts": 1700003600,
            "yes_bid": {
                "open": "0.1000",
                "low": "0.0900",
                "high": "0.1100",
                "close": "0.1000"
            },
            "yes_ask": {
                "open": "0.2000",
                "low": "0.1900",
                "high": "0.2100",
                "close": "0.2000"
            },
            "price": {
                "open": "0.1500",
                "low": "0.1400",
                "high": "0.1600",
                "close": "0.1500",
                "mean": "0.1500",
                "previous": "0.1400"
            },
            "volume": "10.00",
            "open_interest": "20.00"
        }]
    }"#;

    let resp: kalshi_fast::GetMarketCandlesticksHistoricalResponse =
        serde_json::from_str(json).unwrap();
    assert_eq!(resp.ticker, "MKT-1");
    assert_eq!(resp.candlesticks.len(), 1);
    assert_eq!(resp.candlesticks[0].yes_bid.open, "0.1000");
    assert_eq!(resp.candlesticks[0].price.mean.as_deref(), Some("0.1500"));
    assert_eq!(resp.candlesticks[0].volume, "10.00");
}

#[test]
fn get_historical_cutoff_response_deserializes() {
    let json = r#"{
        "market_settled_ts": "2025-01-01T00:00:00Z",
        "trades_created_ts": "2025-01-02T00:00:00Z",
        "orders_updated_ts": "2025-01-03T00:00:00Z"
    }"#;

    let resp: kalshi_fast::GetHistoricalCutoffResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.market_settled_ts, "2025-01-01T00:00:00Z");
    assert_eq!(resp.trades_created_ts, "2025-01-02T00:00:00Z");
    assert_eq!(resp.orders_updated_ts, "2025-01-03T00:00:00Z");
}

#[test]
fn get_positions_response_deserializes() {
    let json = r#"{
        "market_positions": [{
            "ticker": "MKT-1",
            "exchange_index": 0,
            "total_traded_dollars": "12.3400",
            "position_fp": "5.00",
            "market_exposure_dollars": "3.2100",
            "realized_pnl_dollars": "1.1100",
            "fees_paid_dollars": "0.2200",
            "last_updated_ts": "2026-04-16T12:00:00Z"
        }],
        "event_positions": [{
            "event_ticker": "EVT-1",
            "total_cost_dollars": "12.3400",
            "total_cost_shares_fp": "10.00",
            "event_exposure_dollars": "3.2100",
            "realized_pnl_dollars": "1.1100",
            "fees_paid_dollars": "0.2200"
        }],
        "cursor": "abc123"
    }"#;

    let resp: kalshi_fast::GetPositionsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.market_positions.len(), 1);
    assert_eq!(resp.market_positions[0].position_fp, "5.00");
    assert_eq!(resp.market_positions[0].exchange_index, Some(0));
    assert_eq!(resp.event_positions.len(), 1);
    assert_eq!(resp.cursor, Some("abc123".into()));
}

#[test]
fn positions_page_from_response() {
    let json = r#"{
        "market_positions": [{
            "ticker": "MKT-1",
            "exchange_index": 0,
            "total_traded_dollars": "12.3400",
            "position_fp": "5.00",
            "market_exposure_dollars": "3.2100",
            "realized_pnl_dollars": "1.1100",
            "fees_paid_dollars": "0.2200",
            "last_updated_ts": "2026-04-16T12:00:00Z"
        }],
        "event_positions": [{
            "event_ticker": "EVT-1",
            "total_cost_dollars": "12.3400",
            "total_cost_shares_fp": "10.00",
            "event_exposure_dollars": "3.2100",
            "realized_pnl_dollars": "1.1100",
            "fees_paid_dollars": "0.2200"
        }],
        "cursor": "abc123"
    }"#;

    let resp: kalshi_fast::GetPositionsResponse = serde_json::from_str(json).unwrap();
    let page: kalshi_fast::PositionsPage = resp.into();
    assert_eq!(page.market_positions.len(), 1);
    assert_eq!(page.event_positions.len(), 1);
    assert_eq!(page.event_positions[0].total_cost_shares_fp, "10.00");
}

#[test]
fn get_orders_response_deserializes() {
    let json = r#"{
        "orders": [{
            "order_id": "ord-1",
            "user_id": "user-1",
            "client_order_id": "client-1",
            "ticker": "MKT-1",
            "side": "yes",
            "action": "buy",
            "type": "limit",
            "status": "resting",
            "yes_price_dollars": "0.5500",
            "no_price_dollars": "0.4500",
            "fill_count_fp": "0.00",
            "remaining_count_fp": "10.00",
            "initial_count_fp": "10.00",
            "taker_fill_cost_dollars": "0.0000",
            "maker_fill_cost_dollars": "0.0000",
            "taker_fees_dollars": "0.0000",
            "maker_fees_dollars": "0.0000"
        }]
    }"#;

    let resp: kalshi_fast::GetOrdersResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.orders.len(), 1);
    assert_eq!(resp.orders[0].client_order_id, "client-1");
    assert!(resp.cursor.is_none());
}

#[test]
fn create_order_response_deserializes() {
    let json = r#"{
        "order": {
            "order_id": "ord-123",
            "user_id": "user-1",
            "client_order_id": "client-1",
            "ticker": "MKT-1",
            "side": "yes",
            "action": "buy",
            "type": "limit",
            "status": "resting",
            "yes_price_dollars": "0.5500",
            "no_price_dollars": "0.4500",
            "fill_count_fp": "0.00",
            "remaining_count_fp": "10.00",
            "initial_count_fp": "10.00",
            "taker_fill_cost_dollars": "0.0000",
            "maker_fill_cost_dollars": "0.0000",
            "taker_fees_dollars": "0.0000",
            "maker_fees_dollars": "0.0000"
        }
    }"#;

    let resp: kalshi_fast::CreateOrderResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.order.order_id, "ord-123");
    assert_eq!(resp.order.ticker, "MKT-1");
}

#[test]
fn cancel_order_response_deserializes() {
    let json = r#"{
        "order": {
            "order_id": "ord-123",
            "user_id": "user-1",
            "client_order_id": "client-1",
            "ticker": "MKT-1",
            "side": "yes",
            "action": "buy",
            "type": "limit",
            "status": "canceled",
            "yes_price_dollars": "0.5500",
            "no_price_dollars": "0.4500",
            "fill_count_fp": "0.00",
            "remaining_count_fp": "5.00",
            "initial_count_fp": "10.00",
            "taker_fill_cost_dollars": "0.0000",
            "maker_fill_cost_dollars": "0.0000",
            "taker_fees_dollars": "0.0000",
            "maker_fees_dollars": "0.0000"
        },
        "reduced_by": 5,
        "reduced_by_fp": "5.00"
    }"#;

    let resp: kalshi_fast::CancelOrderResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.order.status, OrderStatus::Canceled);
    assert_eq!(resp.reduced_by, 5);
    assert_eq!(resp.reduced_by_fp, "5.00");
}

#[test]
fn get_market_orderbook_response_deserializes() {
    let json = r#"{
        "orderbook_fp": {
            "yes_dollars": [["0.50", "100.00"]],
            "no_dollars": [["0.49", "200.00"]]
        }
    }"#;

    let resp: GetMarketOrderbookResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.orderbook_fp.yes_dollars.len(), 1);
}

#[test]
fn get_trades_response_deserializes() {
    let json = r#"{
        "trades": [{
            "trade_id": "t1",
            "ticker": "MKT-1",
            "count_fp": "2.00",
            "yes_price_dollars": "0.5500",
            "no_price_dollars": "0.4500",
            "taker_side": "yes",
            "taker_outcome_side": "yes",
            "taker_book_side": "bid",
            "created_time": "2026-04-16T12:00:00Z"
        }],
        "cursor": "c1"
    }"#;

    let resp: GetTradesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.trades.len(), 1);
    assert_eq!(resp.trades[0].count_fp, "2.00");
    // Normalized direction fields added 2026-05-07.
    assert!(matches!(
        resp.trades[0].taker_outcome_side,
        Some(TradeTakerSide::Yes)
    ));
    assert!(matches!(
        resp.trades[0].taker_book_side,
        Some(BookSide::Bid)
    ));
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
        "fills": [{
            "fill_id": "f1",
            "order_id": "o1",
            "trade_id": "t1",
            "ticker": "MKT-1",
            "market_ticker": "MKT-1",
            "side": "yes",
            "action": "buy",
            "count_fp": "1.00",
            "yes_price_dollars": "0.5500",
            "no_price_dollars": "0.4500",
            "is_taker": true,
            "fee_cost": "0.0100"
        }],
        "cursor": "c1"
    }"#;

    let resp: GetFillsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.fills.len(), 1);
    assert_eq!(resp.cursor, Some("c1".into()));
}

#[test]
fn get_settlements_response_deserializes() {
    let json = r#"{
        "settlements": [{
            "ticker": "MKT-1",
            "event_ticker": "EVT-1",
            "market_result": "yes",
            "yes_count_fp": "1.00",
            "yes_total_cost_dollars": "0.5500",
            "no_count_fp": "0.00",
            "no_total_cost_dollars": "0.0000",
            "revenue": 100,
            "settled_time": "2026-04-02T00:00:00Z",
            "fee_cost": "0.0100"
        }],
        "cursor": null
    }"#;

    let resp: GetSettlementsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.settlements.len(), 1);
    assert!(resp.cursor.is_none());
}

#[test]
fn get_account_api_limits_response_deserializes() {
    let json = r#"{
        "usage_tier": "basic",
        "read":  {"refill_rate": 20, "bucket_capacity": 200},
        "write": {"refill_rate": 10, "bucket_capacity": 20},
        "grants": [
            {"exchange_instance":"event_contract","level":"premier","source":"volume"},
            {"exchange_instance":"margined","level":"paragon","expires_ts":9999999999,"source":"manual"}
        ]
    }"#;

    let resp: GetAccountApiLimitsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.usage_tier, "basic");
    assert_eq!(resp.read.refill_rate, 20);
    assert_eq!(resp.read.bucket_capacity, 200);
    assert_eq!(resp.write.refill_rate, 10);
    assert_eq!(resp.grants.len(), 2);
    assert_eq!(resp.grants[0].level, "premier");
    assert_eq!(resp.grants[0].source, "volume");
    assert!(resp.grants[0].expires_ts.is_none());
    assert_eq!(resp.grants[1].expires_ts, Some(9999999999));
}

#[test]
fn get_account_api_limits_response_tolerates_missing_grants() {
    // Defensive: a payload without `grants` should still parse (empty vec).
    let json = r#"{
        "usage_tier": "basic",
        "read":  {"refill_rate": 20, "bucket_capacity": 200},
        "write": {"refill_rate": 10, "bucket_capacity": 20}
    }"#;
    let resp: GetAccountApiLimitsResponse = serde_json::from_str(json).unwrap();
    assert!(resp.grants.is_empty());
}

#[test]
fn get_account_endpoint_costs_response_deserializes() {
    let json = r#"{
        "default_cost": 10,
        "endpoint_costs": [
            {"method":"POST","path":"/portfolio/orders","cost":100},
            {"method":"DELETE","path":"/portfolio/orders/batched","cost":50}
        ]
    }"#;

    let resp: GetAccountEndpointCostsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.default_cost, 10);
    assert_eq!(resp.endpoint_costs.len(), 2);
    assert_eq!(resp.endpoint_costs[0].method, "POST");
    assert_eq!(resp.endpoint_costs[0].path, "/portfolio/orders");
    assert_eq!(resp.endpoint_costs[0].cost, 100);

    // Tolerates missing/null endpoint_costs.
    let resp: GetAccountEndpointCostsResponse =
        serde_json::from_str(r#"{"default_cost": 10}"#).unwrap();
    assert!(resp.endpoint_costs.is_empty());
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
        status: Some(MarketStatusQuery::Open),
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
fn get_order_queue_positions_params_allows_unfiltered_requests() {
    assert!(GetOrderQueuePositionsParams::default().validate().is_ok());
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

// ============================================================================
// Optional Field Deserialization Tests (image_url / color_code)
// ============================================================================

#[test]
fn event_metadata_deserializes_with_all_fields() {
    let json = r##"{
        "image_url": "https://example.com/img.png",
        "featured_image_url": "https://example.com/feat.png",
        "market_details": [{"market_ticker": "MKT-1", "image_url": "https://example.com/m.png", "color_code": "#FF0000"}],
        "settlement_sources": [],
        "new_field_from_changelog": "kept"
    }"##;

    let meta: EventMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(
        meta.image_url.as_deref(),
        Some("https://example.com/img.png")
    );
    assert_eq!(
        meta.featured_image_url.as_deref(),
        Some("https://example.com/feat.png")
    );
    assert_eq!(meta.market_details.len(), 1);
    assert_eq!(meta.market_details[0].market_ticker, "MKT-1");
    assert_eq!(
        meta.market_details[0].image_url.as_deref(),
        Some("https://example.com/m.png")
    );
    assert_eq!(
        meta.market_details[0].color_code.as_deref(),
        Some("#FF0000")
    );
    assert_eq!(
        meta.extra
            .get("new_field_from_changelog")
            .and_then(|v| v.as_str()),
        Some("kept")
    );
}

#[test]
fn event_metadata_deserializes_without_image_url() {
    let json = r#"{
        "market_details": [],
        "settlement_sources": []
    }"#;

    let meta: EventMetadata = serde_json::from_str(json).unwrap();
    assert!(meta.image_url.is_none());
    assert!(meta.featured_image_url.is_none());
}

#[test]
fn market_metadata_deserializes_without_image_url() {
    let json = r#"{"market_ticker": "MKT-1"}"#;

    let meta: MarketMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.market_ticker, "MKT-1");
    assert!(meta.image_url.is_none());
    assert!(meta.color_code.is_none());
}

#[test]
fn event_data_deserializes_with_partial_product_metadata() {
    let json = r#"{
        "event_ticker": "EVT-1",
        "product_metadata": {
            "market_details": [],
            "settlement_sources": [],
            "new_product_flag": true
        }
    }"#;

    let event: EventData = serde_json::from_str(json).unwrap();
    assert_eq!(event.event_ticker, "EVT-1");
    let meta = event.product_metadata.unwrap();
    assert!(meta.image_url.is_none());
    assert_eq!(
        meta.extra.get("new_product_flag").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn get_event_response_deserializes_nested_event_markets() {
    let json = r#"{
        "event": {
            "event_ticker": "EVT-1",
            "markets": [{
                "ticker": "MKT-NESTED-1",
                "market_type": "binary",
                "yes_bid_dollars": "0.5600",
                "yes_ask_dollars": "0.5700",
                "volume_fp": "10.00",
                "open_interest_fp": "10.00"
            }]
        },
        "markets": [{
            "ticker": "MKT-TOP-1",
            "market_type": "binary",
            "yes_bid_dollars": "0.6600",
            "yes_ask_dollars": "0.6700",
            "volume_fp": "11.00",
            "open_interest_fp": "11.00"
        }]
    }"#;

    let resp: kalshi_fast::GetEventResponse = serde_json::from_str(json).unwrap();
    let nested = resp.event.markets.as_ref().unwrap();
    assert_eq!(nested.len(), 1);
    assert_eq!(nested[0].ticker, "MKT-NESTED-1");
    assert_eq!(resp.markets.len(), 1);
    assert_eq!(resp.markets[0].ticker, "MKT-TOP-1");
}

#[test]
fn get_event_response_deserializes_without_removed_cent_fields() {
    let json = r#"{
        "event": {
            "event_ticker": "EVT-2",
            "markets": [{
                "ticker": "MKT-2",
                "market_type": "binary",
                "yes_bid_dollars": "0.5600",
                "yes_ask_dollars": "0.5700",
                "volume_fp": "10.00",
                "open_interest_fp": "10.00",
                "notional_value_dollars": "1.0000",
                "liquidity_dollars": "20.0000",
                "price_level_structure": "linear_cent",
                "price_ranges": [{"start":"0.0000","end":"1.0000","step":"0.0100"}]
            }]
        },
        "markets": []
    }"#;

    let resp: kalshi_fast::GetEventResponse = serde_json::from_str(json).unwrap();
    let market = &resp.event.markets.as_ref().unwrap()[0];
    assert_eq!(market.yes_bid, None);
    assert_eq!(market.yes_ask, None);
    assert_eq!(market.notional_value, None);
    assert_eq!(market.yes_bid_dollars.as_deref(), Some("0.5600"));
    assert_eq!(market.liquidity_dollars.as_deref(), Some("20.0000"));
}

#[test]
fn get_api_keys_response_deserializes_typed() {
    let json = r#"{
        "api_keys": [{
            "api_key_id": "key-1",
            "name": "test key",
            "scopes": ["read", "write"]
        }]
    }"#;

    let resp: kalshi_fast::GetApiKeysResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.api_keys.len(), 1);
    assert_eq!(resp.api_keys[0].api_key_id, "key-1");
    assert_eq!(resp.api_keys[0].scopes, vec!["read", "write"]);
}

#[test]
fn quotes_and_rfqs_responses_deserialize_typed() {
    let quotes_json = r#"{
        "quotes": [{
            "id": "q-1",
            "rfq_id": "r-1",
            "creator_id": "u-1",
            "rfq_creator_id": "u-2",
            "market_ticker": "MKT-1",
            "contracts_fp": "10.00",
            "yes_bid_dollars": "0.4400",
            "no_bid_dollars": "0.5600",
            "created_ts": "2023-11-07T05:31:56Z",
            "updated_ts": "2023-11-07T05:31:56Z",
            "status": "open",
            "rfq_target_cost_dollars": "100.0000"
        }]
    }"#;
    let quotes: kalshi_fast::GetQuotesResponse = serde_json::from_str(quotes_json).unwrap();
    assert_eq!(quotes.quotes.len(), 1);
    assert_eq!(quotes.quotes[0].rfq_creator_id, "u-2");
    assert_eq!(
        quotes.quotes[0].rfq_target_cost_dollars.as_deref(),
        Some("100.0000")
    );

    let rfqs_json = r#"{
        "rfqs": [{
            "id": "r-1",
            "creator_id": "u-1",
            "market_ticker": "MKT-1",
            "contracts_fp": "10.00",
            "target_cost_dollars": "100.0000",
            "status": "open",
            "created_ts": "2023-11-07T05:31:56Z"
        }]
    }"#;
    let rfqs: kalshi_fast::GetRFQsResponse = serde_json::from_str(rfqs_json).unwrap();
    assert_eq!(rfqs.rfqs.len(), 1);
    assert_eq!(
        rfqs.rfqs[0].target_cost_dollars.as_deref(),
        Some("100.0000")
    );
}

#[test]
fn multivariate_collections_and_lookup_responses_deserialize_typed() {
    let json = r#"{
        "multivariate_contracts": [{
            "collection_ticker": "COL-1",
            "series_ticker": "SER-1",
            "title": "Collection",
            "description": "Desc",
            "open_date": "2023-11-07T05:31:56Z",
            "close_date": "2023-11-07T05:31:56Z",
            "associated_events": [{
                "ticker": "EVT-1",
                "is_yes_only": false,
                "active_quoters": ["u-1"]
            }],
            "associated_event_tickers": ["EVT-1"],
            "is_ordered": false,
            "is_single_market_per_event": true,
            "is_all_yes": false,
            "size_min": 1,
            "size_max": 2,
            "functional_description": "f(x)"
        }]
    }"#;

    let resp: kalshi_fast::GetMultivariateEventCollectionsResponse =
        serde_json::from_str(json).unwrap();
    assert_eq!(resp.multivariate_contracts.len(), 1);
    assert_eq!(
        resp.multivariate_contracts[0].associated_events[0].ticker,
        "EVT-1"
    );

    let lookup_json = r#"{
        "lookup_points": [{
            "event_ticker": "EVT-1",
            "market_ticker": "MKT-1",
            "selected_markets": [{
                "event_ticker": "EVT-1",
                "market_ticker": "MKT-1",
                "side": "yes"
            }],
            "last_queried_ts": "2023-11-07T05:31:56Z"
        }]
    }"#;
    let lookup: kalshi_fast::GetMultivariateEventCollectionLookupHistoryResponse =
        serde_json::from_str(lookup_json).unwrap();
    assert_eq!(lookup.lookup_points.len(), 1);
    assert_eq!(lookup.lookup_points[0].selected_markets.len(), 1);
}

#[test]
fn batch_order_responses_deserialize_typed() {
    let create_json = r#"{
        "orders": [{
            "client_order_id": "c-1",
            "order": {
                "order_id": "o-1",
                "user_id": "user-1",
                "client_order_id": "c-1",
                "ticker": "MKT-1",
                "side": "yes",
                "action": "buy",
                "type": "limit",
                "status": "resting",
                "yes_price_dollars": "0.5500",
                "no_price_dollars": "0.4500",
                "fill_count_fp": "0.00",
                "remaining_count_fp": "10.00",
                "initial_count_fp": "10.00",
                "taker_fill_cost_dollars": "0.0000",
                "maker_fill_cost_dollars": "0.0000",
                "taker_fees_dollars": "0.0000",
                "maker_fees_dollars": "0.0000"
            },
            "error": null
        }]
    }"#;
    let created: kalshi_fast::BatchCreateOrdersResponse =
        serde_json::from_str(create_json).unwrap();
    assert_eq!(created.orders.len(), 1);
    assert_eq!(
        created.orders[0]
            .order
            .as_ref()
            .map(|o| o.order_id.as_str()),
        Some("o-1")
    );

    let cancel_json = r#"{
        "orders": [{
            "order_id": "o-1",
            "order": {
                "order_id": "o-1",
                "user_id": "user-1",
                "client_order_id": "c-1",
                "ticker": "MKT-1",
                "side": "yes",
                "action": "buy",
                "type": "limit",
                "status": "canceled",
                "yes_price_dollars": "0.5500",
                "no_price_dollars": "0.4500",
                "fill_count_fp": "0.00",
                "remaining_count_fp": "0.00",
                "initial_count_fp": "10.00",
                "taker_fill_cost_dollars": "0.0000",
                "maker_fill_cost_dollars": "0.0000",
                "taker_fees_dollars": "0.0000",
                "maker_fees_dollars": "0.0000"
            },
            "reduced_by": 1,
            "reduced_by_fp": "1.00",
            "error": null
        }]
    }"#;
    let canceled: kalshi_fast::BatchCancelOrdersResponse =
        serde_json::from_str(cancel_json).unwrap();
    assert_eq!(canceled.orders.len(), 1);
    assert_eq!(canceled.orders[0].reduced_by_fp, "1.00");
}

#[test]
fn fills_deserialize_current_schema() {
    let json = r#"{
        "fills": [{
            "fill_id": "f-1",
            "order_id": "o-1",
            "trade_id": "t-1",
            "ticker": "MKT-1",
            "market_ticker": "MKT-1",
            "side": "yes",
            "action": "buy",
            "count_fp": "1.00",
            "yes_price_dollars": "0.5500",
            "no_price_dollars": "0.4500",
            "is_taker": true,
            "fee_cost": "0.0100",
            "ts": 1771113600
        }]
    }"#;
    let current: kalshi_fast::GetFillsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(current.fills[0].yes_price_dollars, "0.5500");
    assert_eq!(current.fills[0].no_price_dollars, "0.4500");
    assert_eq!(current.fills[0].ts, Some(1771113600));
}

#[test]
fn settlements_deserialize_current_schema() {
    let json = r#"{
        "settlements": [{
            "ticker": "MKT-1",
            "event_ticker": "EVT-1",
            "market_result": "yes",
            "yes_count_fp": "1.00",
            "yes_total_cost_dollars": "0.5500",
            "no_count_fp": "0.00",
            "no_total_cost_dollars": "0.0000",
            "revenue": 100,
            "settled_time": "2026-04-02T00:00:00Z",
            "fee_cost": "0.0100",
            "value": 99
        }]
    }"#;
    let current: kalshi_fast::GetSettlementsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(current.settlements[0].yes_total_cost_dollars, "0.5500");
    assert_eq!(current.settlements[0].no_total_cost_dollars, "0.0000");
    assert_eq!(current.settlements[0].revenue, 100);
    assert_eq!(current.settlements[0].value, Some(99));
}

#[test]
fn queue_positions_forecast_and_structured_targets_deserialize_typed() {
    let queue_json = r#"{
        "queue_positions": [{
            "order_id": "o-1",
            "market_ticker": "MKT-1",
            "queue_position": 4,
            "queue_position_fp": "4.00"
        }]
    }"#;
    let queue: kalshi_fast::GetOrderQueuePositionsResponse =
        serde_json::from_str(queue_json).unwrap();
    assert_eq!(queue.queue_positions.len(), 1);
    assert_eq!(
        queue.queue_positions[0].queue_position_fp.as_deref(),
        Some("4.00")
    );

    let forecast_json = r#"{
        "forecast_history": [{
            "event_ticker": "EVT-1",
            "end_period_ts": 123,
            "period_interval": 60,
            "percentile_points": [{
                "percentile": 5000,
                "raw_numerical_forecast": 12.3,
                "numerical_forecast": 12.3,
                "formatted_forecast": "12.3"
            }]
        }]
    }"#;
    let forecast: kalshi_fast::GetEventForecastPercentilesHistoryResponse =
        serde_json::from_str(forecast_json).unwrap();
    assert_eq!(forecast.forecast_history.len(), 1);
    assert_eq!(forecast.forecast_history[0].percentile_points.len(), 1);

    let targets_json = r#"{
        "structured_targets": [{
            "id": "st-1",
            "name": "Target 1",
            "type": "politics",
            "details": {"k":"v"},
            "source_id": "source-1",
            "source_ids": {"alt":"a-1"},
            "last_updated_ts": "2023-11-07T05:31:56Z"
        }]
    }"#;
    let targets: kalshi_fast::GetStructuredTargetsResponse =
        serde_json::from_str(targets_json).unwrap();
    assert_eq!(targets.structured_targets.len(), 1);
    assert_eq!(
        targets.structured_targets[0].target_type.as_deref(),
        Some("politics")
    );
}
