mod common;

use kalshi::{
    EventStatus, GetEventsParams, GetMarketsParams, GetSeriesListParams, KalshiRestClient,
    MarketStatus,
};

#[tokio::test]
async fn test_get_series_list() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_series_list(GetSeriesListParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Series list should be non-empty on demo
    assert!(!resp.series.is_empty());
}

#[tokio::test]
async fn test_get_events() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
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

    assert!(resp.events.len() <= 5);
}

#[tokio::test]
async fn test_get_event_by_ticker() {
    let client = KalshiRestClient::new(common::demo_env());

    // First get an event ticker from the events list
    let events_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_events(GetEventsParams {
                limit: Some(1),
                status: Some(EventStatus::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if events_resp.events.is_empty() {
        // No open events on demo, skip this test
        return;
    }

    let event_ticker = events_resp.events[0]["event_ticker"]
        .as_str()
        .expect("event_ticker should be a string");

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_event(event_ticker, Some(true)).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.event.is_object());
}

#[tokio::test]
async fn test_get_markets() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets(GetMarketsParams {
                limit: Some(5),
                status: Some(MarketStatus::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.markets.len() <= 5);
}

#[tokio::test]
async fn test_get_market_by_ticker() {
    let client = KalshiRestClient::new(common::demo_env());

    // First get a market ticker from the markets list
    let markets_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
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
        // No open markets on demo, skip this test
        return;
    }

    let market_ticker = markets_resp.markets[0]["ticker"]
        .as_str()
        .expect("ticker should be a string");

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_market(market_ticker).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.market.is_object());
}
