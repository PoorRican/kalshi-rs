mod common;

use kalshi::{
    EventStatus, GetEventsParams, GetMarketsParams, GetSeriesFeeChangesParams, GetSeriesListParams,
    GetTradesParams, KalshiRestClient, MarketStatus,
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
async fn test_get_series_by_ticker() {
    let client = KalshiRestClient::new(common::demo_env());
    let list_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_series_list(GetSeriesListParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if list_resp.series.is_empty() {
        return;
    }

    let ticker = list_resp.series[0].ticker.clone();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_series(&ticker).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(resp.series.ticker, ticker);
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

    let event_ticker = events_resp.events[0].event_ticker.clone();

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_event(&event_ticker, Some(true)).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(resp.event.event_ticker, event_ticker);
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

    let market_ticker = markets_resp.markets[0].ticker.clone();

    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_market(&market_ticker).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(resp.market.ticker, market_ticker);
}

#[tokio::test]
async fn test_get_market_orderbook() {
    let client = KalshiRestClient::new(common::demo_env());

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
        return;
    }

    let market_ticker = markets_resp.markets[0].ticker.clone();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_market_orderbook(&market_ticker, Some(1)).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.orderbook.yes.len() <= 1);
    assert!(resp.orderbook.no.len() <= 1);
}

#[tokio::test]
async fn test_get_trades() {
    let client = KalshiRestClient::new(common::demo_env());

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
        return;
    }

    let market_ticker = markets_resp.markets[0].ticker.clone();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_trades(GetTradesParams {
                ticker: Some(market_ticker),
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.trades.len() <= 1);
}

#[tokio::test]
async fn test_get_exchange_status() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_exchange_status().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(ts) = resp.exchange_estimated_resume_time.as_deref() {
        assert!(!ts.is_empty());
    }
}

#[tokio::test]
async fn test_get_exchange_announcements() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_exchange_announcements().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = resp.announcements.first() {
        assert!(!first.message.is_empty());
    }
}

#[tokio::test]
async fn test_get_exchange_schedule() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_exchange_schedule().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = resp.schedule.standard_hours.first() {
        assert!(!first.start_time.is_empty());
        assert!(!first.end_time.is_empty());
    }
}

#[tokio::test]
async fn test_get_user_data_timestamp() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_user_data_timestamp().await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(!resp.as_of_time.is_empty());
}

#[tokio::test]
async fn test_get_series_fee_changes() {
    let client = KalshiRestClient::new(common::demo_env());
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_series_fee_changes(GetSeriesFeeChangesParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = resp.series_fee_change_arr.first() {
        assert!(first.scheduled_ts > 0);
    }
}
