#![cfg(feature = "live-tests")]

mod common;

use kalshi_fast::{
    EventStatus, GetBalanceParams, GetEventForecastPercentileHistoryParams, GetEventsParams,
    GetFillsParams, GetOrderQueuePositionsParams, GetOrdersParams, GetPositionsParams,
    GetQuotesParams, GetRFQsParams, GetSettlementsParams, GetSubaccountTransfersParams,
    KalshiError, OrderStatus, SubaccountQueryParams,
};
use reqwest::StatusCode;

#[tokio::test]
async fn test_get_balance() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_balance(GetBalanceParams::default()).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Balance fields should exist (may be 0)
    assert!(resp.balance >= 0);
    assert!(resp.portfolio_value >= 0);
    assert!(resp.updated_ts > 0);
}

#[tokio::test]
async fn test_get_positions() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_positions(GetPositionsParams {
                limit: Some(10),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Positions may be empty, but the vectors should exist
    assert!(resp.market_positions.len() <= 10);
}

#[tokio::test]
async fn test_get_orders() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_orders(GetOrdersParams {
                limit: Some(10),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Orders may be empty, but the vector should exist
    assert!(resp.orders.len() <= 10);
}

#[tokio::test]
async fn test_get_fills() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_fills(GetFillsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.fills.len() <= 1);
}

#[tokio::test]
async fn test_get_settlements() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_settlements(GetSettlementsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.settlements.len() <= 1);
}

#[tokio::test]
async fn test_get_account_api_limits() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_account_api_limits().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.read.bucket_capacity >= 0);
    assert!(resp.write.bucket_capacity >= 0);
}

#[tokio::test]
async fn test_get_subaccount_balances() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_subaccount_balances().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = resp.subaccount_balances.first() {
        assert!(first.updated_ts > 0);
    }
}

#[tokio::test]
async fn test_get_subaccount_transfers() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_subaccount_transfers(GetSubaccountTransfersParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.subaccount_transfers.len() <= 1);
}

#[tokio::test]
async fn test_auth_required_without_auth() {
    let client = common::demo_client();

    let result = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_balance(GetBalanceParams::default()).await
    })
    .await;

    match result {
        Ok(Err(KalshiError::AuthRequired(_))) => {
            // Expected: auth required error from client
        }
        Ok(Err(e)) => panic!("Expected AuthRequired, got: {:?}", e),
        Ok(Ok(_)) => panic!("Expected error, got success"),
        Err(_) => panic!("timeout"),
    }
}

#[tokio::test]
async fn test_get_portfolio_total_resting_order_value() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_portfolio_total_resting_order_value().await
    })
    .await
    .expect("timeout");

    match resp {
        Ok(resp) => assert!(resp.total_resting_order_value >= 0),
        Err(KalshiError::Http {
            status, api_error, ..
        }) => {
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert_eq!(
                api_error.and_then(|error| error.code),
                Some("permission_denied".to_string())
            );
        }
        Err(err) => panic!("unexpected error: {err:?}"),
    }
}

#[tokio::test]
async fn test_get_api_keys() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async { client.get_api_keys().await })
        .await
        .expect("timeout")
        .expect("request failed");
}

#[tokio::test]
async fn test_get_communications_id() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_communications_id().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(!resp.communications_id.is_empty());
}

#[tokio::test]
async fn test_get_order_groups() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_order_groups(SubaccountQueryParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_order_queue_positions() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resting_orders = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_orders(GetOrdersParams {
                limit: Some(10),
                status: Some(OrderStatus::Resting),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let Some(first_order) = resting_orders.orders.first() else {
        return;
    };

    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_order_queue_positions(GetOrderQueuePositionsParams {
                market_tickers: Some(first_order.ticker.clone()),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_rfqs() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_rfqs(GetRFQsParams::default()).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // RFQs may be empty, but the response should parse
    let _ = resp.rfqs;
}

#[tokio::test]
async fn test_get_quotes() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let rfqs = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_rfqs(GetRFQsParams {
                limit: Some(10),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let Some(first_rfq) = rfqs.rfqs.iter().find(|rfq| {
        rfq.creator_user_id
            .as_deref()
            .is_some_and(|id| !id.is_empty())
    }) else {
        return;
    };

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_quotes(GetQuotesParams {
                rfq_id: Some(first_rfq.id.clone()),
                rfq_creator_user_id: first_rfq.creator_user_id.clone(),
                limit: Some(10),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Quotes may be empty, but the response should parse
    let _ = resp.quotes;
}

#[tokio::test]
async fn test_get_event_forecast_percentile_history() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    // First find an open event with a series_ticker
    let events_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_events(GetEventsParams {
                limit: Some(5),
                status: Some(EventStatus::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut saw_bad_request = false;

    for event in events_resp
        .events
        .iter()
        .filter(|e| e.series_ticker.is_some())
    {
        let series_ticker = event.series_ticker.as_ref().expect("checked above").clone();
        let event_ticker = event.event_ticker.clone();

        let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
            client
                .get_event_forecast_percentile_history(
                    &series_ticker,
                    &event_ticker,
                    GetEventForecastPercentileHistoryParams {
                        percentiles: vec![2500, 5000, 7500],
                        start_ts: now - 86400 * 7,
                        end_ts: now,
                        period_interval: 60,
                    },
                )
                .await
        })
        .await
        .expect("timeout");

        match resp {
            Ok(_) => return,
            Err(KalshiError::Http { status, .. }) if status == StatusCode::BAD_REQUEST => {
                saw_bad_request = true;
            }
            Err(err) => panic!("request failed: {err:?}"),
        }
    }

    if saw_bad_request {
        return;
    }
}

#[tokio::test]
async fn test_get_subaccount_transfers_all() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let transfers = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_subaccount_transfers_all(GetSubaccountTransfersParams {
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Transfers may be empty on demo
    let _ = transfers;
}

#[tokio::test]
async fn test_get_rfqs_all() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let first_page = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_rfqs(GetRFQsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let Some(first_rfq) = first_page.rfqs.first() else {
        return;
    };

    let rfqs = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_rfqs_all(GetRFQsParams {
                market_ticker: Some(first_rfq.market_ticker.clone()),
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let _ = rfqs;
}

#[tokio::test]
async fn test_get_quotes_all() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let rfqs = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_rfqs(GetRFQsParams {
                limit: Some(10),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let Some(first_rfq) = rfqs.rfqs.iter().find(|rfq| {
        rfq.creator_user_id
            .as_deref()
            .is_some_and(|id| !id.is_empty())
    }) else {
        return;
    };

    let quotes = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_quotes_all(GetQuotesParams {
                rfq_id: Some(first_rfq.id.clone()),
                rfq_creator_user_id: first_rfq.creator_user_id.clone(),
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let _ = quotes;
}

#[tokio::test]
async fn test_get_subaccount_netting() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_subaccount_netting().await
    })
    .await
    .expect("timeout");

    match resp {
        Ok(resp) => {
            let _ = resp.netting_configs;
        }
        Err(KalshiError::Http {
            status, api_error, ..
        }) => {
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            let api_error = api_error.expect("expected api error");
            assert_eq!(api_error.code.as_deref(), Some("internal_server_error"));
        }
        Err(err) => panic!("unexpected error: {err:?}"),
    }
}
