mod common;

use kalshi::{
    GetMarketsParams, KalshiRestClient, KalshiWsClient, MarketStatus, WsChannel, WsDataMessage,
    WsMessage, WsSubscriptionParams,
};
use std::time::Duration;

#[tokio::test]
async fn test_ws_authenticated_connect() {
    common::load_env();
    let auth = common::load_auth();

    let ws = tokio::time::timeout(common::TEST_TIMEOUT, async {
        KalshiWsClient::connect_authenticated(common::demo_env(), auth).await
    })
    .await
    .expect("timeout")
    .expect("connection failed");

    // Connection succeeded with auth
    drop(ws);
}

#[tokio::test]
async fn test_ws_orderbook_delta_subscribe() {
    common::load_env();
    let auth = common::load_auth();

    // First get an open market ticker via REST
    let rest_client = KalshiRestClient::new(common::demo_env());
    let markets_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        rest_client
            .get_markets(GetMarketsParams {
                limit: Some(1),
                status: Some(MarketStatus::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if markets_resp.markets.is_empty() {
        // No open markets, skip test
        return;
    }
    let market_ticker = markets_resp.markets[0].ticker.clone();

    // Connect with auth
    let mut ws = tokio::time::timeout(common::TEST_TIMEOUT, async {
        KalshiWsClient::connect_authenticated(common::demo_env(), auth).await
    })
    .await
    .expect("timeout")
    .expect("connection failed");

    // Subscribe to OrderbookDelta (private channel) with market ticker
    let sub_id = ws
        .subscribe(WsSubscriptionParams {
            channels: vec![WsChannel::OrderbookDelta],
            market_tickers: Some(vec![market_ticker]),
            ..Default::default()
        })
        .await
        .expect("subscribe failed");

    assert!(sub_id > 0);

    // Read first message
    let msg = tokio::time::timeout(Duration::from_secs(10), async { ws.next_message().await })
        .await
        .expect("timeout")
        .expect("receive failed");

    match msg {
        WsMessage::Subscribed { .. } => {}
        WsMessage::Data(WsDataMessage::OrderbookDelta { .. }) => {}
        WsMessage::Data(WsDataMessage::OrderbookSnapshot { .. }) => {}
        other => panic!("unexpected message: {:?}", other),
    }
}

#[tokio::test]
async fn test_ws_fill_subscribe() {
    common::load_env();
    let auth = common::load_auth();

    let mut ws = tokio::time::timeout(common::TEST_TIMEOUT, async {
        KalshiWsClient::connect_authenticated(common::demo_env(), auth).await
    })
    .await
    .expect("timeout")
    .expect("connection failed");

    // Subscribe to Fill channel (private, no market ticker needed)
    let sub_id = ws
        .subscribe(WsSubscriptionParams {
            channels: vec![WsChannel::Fill],
            ..Default::default()
        })
        .await
        .expect("subscribe failed");

    assert!(sub_id > 0);

    // Read first message (should be subscribed confirmation)
    let msg = tokio::time::timeout(Duration::from_secs(10), async { ws.next_message().await })
        .await
        .expect("timeout")
        .expect("receive failed");

    match msg {
        WsMessage::Subscribed { .. } => {}
        WsMessage::Data(WsDataMessage::Fill { .. }) => {}
        other => panic!("unexpected message: {:?}", other),
    }
}
