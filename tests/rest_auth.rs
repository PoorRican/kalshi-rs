mod common;

use kalshi_fast::{
    GetFillsParams, GetOrdersParams, GetPositionsParams, GetSettlementsParams,
    GetSubaccountTransfersParams, KalshiError, KalshiRestClient,
};

#[tokio::test]
async fn test_get_balance() {
    common::load_env();
    let auth = common::load_auth();
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async { client.get_balance().await })
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
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

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
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

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
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

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
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

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
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_account_api_limits().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.read_limit >= 0);
    assert!(resp.write_limit >= 0);
}

#[tokio::test]
async fn test_get_subaccount_balances() {
    common::load_env();
    let auth = common::load_auth();
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

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
    let client = KalshiRestClient::new(common::demo_env()).with_auth(auth);

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
    let client = KalshiRestClient::new(common::demo_env());

    let result =
        tokio::time::timeout(common::TEST_TIMEOUT, async { client.get_balance().await }).await;

    match result {
        Ok(Err(KalshiError::AuthRequired(_))) => {
            // Expected: auth required error from client
        }
        Ok(Err(e)) => panic!("Expected AuthRequired, got: {:?}", e),
        Ok(Ok(_)) => panic!("Expected error, got success"),
        Err(_) => panic!("timeout"),
    }
}
