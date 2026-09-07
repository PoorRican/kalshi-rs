#![cfg(feature = "live-tests")]

mod common;

use kalshi_fast::{
    AmendOrderV2Request, BatchCancelOrderV2RequestOrder, BatchCancelOrdersV2Request,
    BatchCreateOrdersV2Request, BookSide, CancelOrderV2Params, CreateOrderV2Request,
    GetMarketsParams, GetOrdersParams, MarketStatusQuery, SelfTradePreventionType,
    SubaccountQueryParams, TimeInForce,
};
use std::time::Duration;

/// Longer timeout for multi-step lifecycle tests
const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_order_lifecycle() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    // 1. Find an open market
    let markets_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets(GetMarketsParams {
                limit: Some(1),
                status: Some(MarketStatusQuery::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if markets_resp.markets.is_empty() {
        return;
    }

    let market_ticker = markets_resp.markets[0].ticker.clone();

    // 2. Create a limit order at an extreme price so it won't fill (1 cent YES)
    let create_resp = tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        client
            .create_order_v2(CreateOrderV2Request {
                ticker: market_ticker.clone(),
                side: BookSide::Bid, // bid == yes
                count: "1".to_string(),
                price: "0.01".to_string(),
                time_in_force: TimeInForce::GoodTillCanceled,
                self_trade_prevention_type: SelfTradePreventionType::TakerAtCross,
                client_order_id: None,
                expiration_time: None,
                post_only: None,
                cancel_order_on_pause: None,
                reduce_only: None,
                subaccount: None,
                order_group_id: None,
                exchange_index: None,
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("create_order_v2 failed");

    let order_id = create_resp.order_id.clone();

    // Use a closure to ensure cleanup even on assertion failures
    let result = async {
        // 3. Get order by ID and verify
        let get_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
            client.get_order(&order_id).await
        })
        .await
        .expect("timeout")
        .expect("get_order failed");

        assert_eq!(get_resp.order.order_id, order_id);
        assert_eq!(get_resp.order.ticker, market_ticker);

        // 4. Amend the order (change price to 2 cents)
        let amend_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
            client
                .amend_order_v2(
                    &order_id,
                    SubaccountQueryParams::default(),
                    AmendOrderV2Request {
                        ticker: market_ticker.clone(),
                        side: BookSide::Bid,
                        price: "0.02".to_string(),
                        count: "1".to_string(),
                        client_order_id: None,
                        updated_client_order_id: None,
                        exchange_index: None,
                    },
                )
                .await
        })
        .await
        .expect("timeout")
        .expect("amend_order_v2 failed");

        assert_eq!(amend_resp.order_id, order_id);

        // 5. Get queue position for the order
        let _queue_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
            client.get_order_queue_position(&amend_resp.order_id).await
        })
        .await
        .expect("timeout")
        .expect("get_order_queue_position failed");

        amend_resp.order_id.clone()
    }
    .await;

    // 6. Cancel the order (cleanup) - use the potentially amended order_id
    let cancel_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .cancel_order_v2(&result, CancelOrderV2Params::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("cancel_order_v2 failed");

    assert_eq!(cancel_resp.order_id, result);

    // 7. Verify order is cancelled via get_orders
    let orders_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_orders(GetOrdersParams {
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("get_orders failed");

    // The cancelled order should not appear in active orders
    assert!(!orders_resp.orders.iter().any(|o| o.order_id == result));
}

#[tokio::test]
async fn test_batch_order_lifecycle() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    // 1. Find an open market
    let markets_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets(GetMarketsParams {
                limit: Some(1),
                status: Some(MarketStatusQuery::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if markets_resp.markets.is_empty() {
        return;
    }

    let market_ticker = markets_resp.markets[0].ticker.clone();

    let order_template = |price: &str| CreateOrderV2Request {
        ticker: market_ticker.clone(),
        side: BookSide::Bid,
        count: "1".to_string(),
        price: price.to_string(),
        time_in_force: TimeInForce::GoodTillCanceled,
        self_trade_prevention_type: SelfTradePreventionType::TakerAtCross,
        client_order_id: None,
        expiration_time: None,
        post_only: None,
        cancel_order_on_pause: None,
        reduce_only: None,
        subaccount: None,
        order_group_id: None,
        exchange_index: None,
    };

    // 2. Batch create 2 limit orders at extreme prices
    let batch_resp = tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        client
            .batch_create_orders_v2(BatchCreateOrdersV2Request {
                orders: vec![order_template("0.01"), order_template("0.02")],
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("batch_create_orders_v2 failed");

    let order_ids: Vec<String> = batch_resp
        .orders
        .iter()
        .filter_map(|r| r.order_id.clone())
        .collect();

    assert!(
        !order_ids.is_empty(),
        "at least one order should have been created"
    );

    // 3. Batch cancel all created orders (cleanup)
    let cancel_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .batch_cancel_orders_v2(BatchCancelOrdersV2Request {
                orders: order_ids
                    .iter()
                    .map(|id| BatchCancelOrderV2RequestOrder {
                        order_id: id.clone(),
                        subaccount: None,
                        exchange_index: None,
                    })
                    .collect(),
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("batch_cancel_orders_v2 failed");

    assert_eq!(cancel_resp.orders.len(), order_ids.len());
}
