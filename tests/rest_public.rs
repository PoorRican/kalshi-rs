#![cfg(feature = "live-tests")]

mod common;

use kalshi_fast::{
    BatchGetMarketCandlesticksParams, EventStatus, GetEventCandlesticksParams, GetEventsParams,
    GetIncentiveProgramsParams, GetMarketCandlesticksParams, GetMarketsParams, GetMilestonesParams,
    GetMultivariateEventCollectionLookupHistoryParams, GetMultivariateEventCollectionsParams,
    GetMultivariateEventsParams, GetSeriesFeeChangesParams, GetSeriesListParams,
    GetStructuredTargetsParams, GetTradesParams, MarketStatusQuery,
};

fn assert_market_list_shape(market: &kalshi_fast::Market) {
    assert!(!market.ticker.is_empty());
    assert!(
        market
            .event_ticker
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(market.status.is_some());
    assert!(
        market
            .volume_fp
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        market
            .volume_24h_fp
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        market
            .open_interest_fp
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    );
}

#[tokio::test]
async fn test_get_series_list() {
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_series_list(GetSeriesListParams::default()).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    // Series list should be non-empty on demo
    assert!(!resp.series.is_empty());
}

#[tokio::test]
async fn test_get_series_by_ticker() {
    let client = common::demo_client();
    let list_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_series_list(GetSeriesListParams::default()).await
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
    let client = common::demo_client();
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
async fn test_get_events_all() {
    let client = common::demo_client();
    let events = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_events_all(GetEventsParams {
                // Use a far-future close filter to keep the result set small.
                min_close_ts: Some(4_102_444_800),
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = events.first() {
        assert!(!first.event_ticker.is_empty());
    }
}

#[tokio::test]
async fn test_get_event_by_ticker() {
    let client = common::demo_client();

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
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets(GetMarketsParams {
                limit: Some(5),
                status: Some(MarketStatusQuery::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if resp.markets.is_empty() {
        return;
    }

    assert!(resp.markets.len() <= 5);
    for market in &resp.markets {
        assert_market_list_shape(market);
    }
}

#[tokio::test]
async fn test_get_market_by_ticker() {
    let client = common::demo_client();

    // First get a market ticker from the markets list
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
    let client = common::demo_client();

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
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_market_orderbook(&market_ticker, Some(1)).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.orderbook_fp.yes_dollars.len() <= 1);
    assert!(resp.orderbook_fp.no_dollars.len() <= 1);
}

#[tokio::test]
async fn test_get_trades() {
    let client = common::demo_client();

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
    let client = common::demo_client();
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
async fn test_get_exchange_schedule() {
    let client = common::demo_client();
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
    let client = common::demo_client();
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
    let client = common::demo_client();
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

#[tokio::test]
async fn test_get_multivariate_events() {
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_events(GetMultivariateEventsParams {
                limit: Some(5),
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
async fn test_get_event_metadata() {
    let client = common::demo_client();

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
        return;
    }

    let event_ticker = events_resp.events[0].event_ticker.clone();
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_event_metadata(&event_ticker).await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_milestones() {
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_milestones(GetMilestonesParams {
                limit: Some(5),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.milestones.len() <= 5);
}

#[tokio::test]
async fn test_get_milestone() {
    let client = common::demo_client();

    let milestones_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_milestones(GetMilestonesParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if milestones_resp.milestones.is_empty() {
        return;
    }

    let milestone_id = match milestones_resp.milestones[0].id.as_deref() {
        Some(id) => id.to_string(),
        None => return,
    };
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_milestone(&milestone_id).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(resp.milestone.id.as_deref(), Some(milestone_id.as_str()));
}

#[tokio::test]
async fn test_get_multivariate_event_collections() {
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_event_collections(GetMultivariateEventCollectionsParams {
                limit: Some(5),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.multivariate_contracts.len() <= 5);
}

#[tokio::test]
async fn test_get_multivariate_event_collection() {
    let client = common::demo_client();

    let collections_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_event_collections(GetMultivariateEventCollectionsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if collections_resp.multivariate_contracts.is_empty() {
        return;
    }

    let ticker = collections_resp.multivariate_contracts[0]
        .collection_ticker
        .clone();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_multivariate_event_collection(&ticker).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(resp.multivariate_contract.collection_ticker, ticker);
}

#[tokio::test]
async fn test_get_multivariate_event_collection_lookup_history() {
    let client = common::demo_client();

    let collections_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_event_collections(GetMultivariateEventCollectionsParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if collections_resp.multivariate_contracts.is_empty() {
        return;
    }

    let ticker = collections_resp.multivariate_contracts[0]
        .collection_ticker
        .clone();
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_event_collection_lookup_history(
                &ticker,
                GetMultivariateEventCollectionLookupHistoryParams::default(),
            )
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_structured_targets() {
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_structured_targets(GetStructuredTargetsParams {
                page_size: Some(5),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert!(resp.structured_targets.len() <= 5);
}

#[tokio::test]
async fn test_get_structured_target() {
    let client = common::demo_client();

    let targets_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_structured_targets(GetStructuredTargetsParams {
                page_size: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if targets_resp.structured_targets.is_empty() {
        return;
    }

    let target_id = match targets_resp.structured_targets[0].id.as_deref() {
        Some(id) => id.to_string(),
        None => return,
    };
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_structured_target(&target_id).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(
        resp.structured_target.id.as_deref(),
        Some(target_id.as_str())
    );
}

#[tokio::test]
async fn test_get_tags_by_categories() {
    let client = common::demo_client();
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_tags_by_categories().await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_filters_by_sport() {
    let client = common::demo_client();
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_filters_by_sport().await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_batch_get_market_candlesticks() {
    let client = common::demo_client();

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

    let ticker = markets_resp.markets[0].ticker.clone();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .batch_get_market_candlesticks(BatchGetMarketCandlesticksParams {
                market_tickers: ticker,
                start_ts: now - 86400,
                end_ts: now,
                period_interval: 60,
                include_latest_before_start: None,
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_market_candlesticks() {
    let client = common::demo_client();

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

    let market = &markets_resp.markets[0];
    let series_ticker = match &market.series_ticker {
        Some(st) => st.clone(),
        None => return,
    };
    let ticker = market.ticker.clone();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_market_candlesticks(
                &series_ticker,
                &ticker,
                GetMarketCandlesticksParams {
                    start_ts: now - 86400,
                    end_ts: now,
                    period_interval: 60,
                    include_latest_before_start: None,
                },
            )
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_event_market_candlesticks() {
    let client = common::demo_client();

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
        return;
    }

    let event = &events_resp.events[0];
    let series_ticker = match &event.series_ticker {
        Some(st) => st.clone(),
        None => return,
    };
    let event_ticker = event.event_ticker.clone();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_event_market_candlesticks(
                &series_ticker,
                &event_ticker,
                GetEventCandlesticksParams {
                    start_ts: now - 86400,
                    end_ts: now,
                    period_interval: 60,
                },
            )
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_incentive_programs() {
    let client = common::demo_client();
    // May not be available on demo - just verify the endpoint doesn't panic
    let _resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_incentive_programs(GetIncentiveProgramsParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");
}

#[tokio::test]
async fn test_get_markets_all() {
    let client = common::demo_client();
    let events = tokio::time::timeout(common::TEST_TIMEOUT, async {
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

    let Some(first_event) = events.events.first() else {
        return;
    };

    let markets = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets_all(GetMarketsParams {
                limit: Some(100),
                event_ticker: Some(vec![first_event.event_ticker.clone()]),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if markets.is_empty() {
        return;
    }

    for market in &markets {
        assert_market_list_shape(market);
        assert_eq!(
            market.event_ticker.as_deref(),
            Some(first_event.event_ticker.as_str())
        );
    }
}

#[tokio::test]
async fn test_get_markets_can_roundtrip_event_filter_for_open_market() {
    let client = common::demo_client();
    let resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets(GetMarketsParams {
                limit: Some(25),
                status: Some(MarketStatusQuery::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let Some(seed_market) = resp
        .markets
        .into_iter()
        .find(|market| market.event_ticker.is_some())
    else {
        return;
    };

    assert_market_list_shape(&seed_market);

    let event_ticker = seed_market.event_ticker.clone().unwrap();
    let event_markets = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_markets(GetMarketsParams {
                limit: Some(100),
                event_ticker: Some(vec![event_ticker.clone()]),
                status: Some(MarketStatusQuery::Open),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if event_markets.markets.is_empty() {
        return;
    }

    for market in &event_markets.markets {
        assert_market_list_shape(market);
        assert_eq!(market.event_ticker.as_deref(), Some(event_ticker.as_str()));
    }
}

#[tokio::test]
async fn test_get_trades_all() {
    let client = common::demo_client();

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

    let ticker = markets_resp.markets[0].ticker.clone();
    let trades = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_trades_all(GetTradesParams {
                ticker: Some(ticker),
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = trades.first() {
        assert!(!first.ticker.is_empty());
    }
}

#[tokio::test]
async fn test_get_milestones_all() {
    let client = common::demo_client();
    let first_page = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_milestones(GetMilestonesParams {
                limit: Some(5),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let params = if let Some(related_event_ticker) = first_page
        .milestones
        .iter()
        .find_map(|milestone| milestone.related_event_tickers.first().cloned())
    {
        GetMilestonesParams {
            limit: Some(100),
            related_event_ticker: Some(related_event_ticker),
            ..Default::default()
        }
    } else if let Some(source_id) = first_page
        .milestones
        .iter()
        .find_map(|milestone| milestone.source_id.clone())
    {
        GetMilestonesParams {
            limit: Some(100),
            source_id: Some(source_id),
            ..Default::default()
        }
    } else {
        return;
    };

    let milestones = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_milestones_all(params).await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = milestones.first() {
        assert!(first.id.is_some());
    }
}

#[tokio::test]
async fn test_get_multivariate_events_all() {
    let client = common::demo_client();
    let collections = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_event_collections(GetMultivariateEventCollectionsParams {
                limit: Some(10),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let mut selected_collection = None;
    for collection in &collections.multivariate_contracts {
        let collection_ticker = collection.collection_ticker.clone();
        let page = tokio::time::timeout(common::TEST_TIMEOUT, async {
            client
                .get_multivariate_events(GetMultivariateEventsParams {
                    collection_ticker: Some(collection_ticker.clone()),
                    limit: Some(1),
                    ..Default::default()
                })
                .await
        })
        .await
        .expect("timeout")
        .expect("request failed");

        if page.cursor.as_deref().unwrap_or_default().is_empty() {
            selected_collection = Some(collection_ticker);
            break;
        }
    }

    let Some(collection_ticker) = selected_collection else {
        return;
    };

    let events = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_events_all(GetMultivariateEventsParams {
                limit: Some(100),
                collection_ticker: Some(collection_ticker),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = events.first() {
        assert!(!first.event_ticker.is_empty());
    }
}

#[tokio::test]
async fn test_get_multivariate_event_collections_all() {
    let client = common::demo_client();
    let collections = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_multivariate_event_collections_all(GetMultivariateEventCollectionsParams {
                limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = collections.first() {
        assert!(!first.collection_ticker.is_empty());
    }
}

#[tokio::test]
async fn test_get_structured_targets_all() {
    let client = common::demo_client();
    let first_page = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_structured_targets(GetStructuredTargetsParams {
                page_size: Some(1),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    let Some(first_id) = first_page
        .structured_targets
        .first()
        .and_then(|target| target.id.clone())
    else {
        return;
    };

    let targets = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_structured_targets_all(GetStructuredTargetsParams {
                ids: Some(vec![first_id]),
                page_size: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("request failed");

    if let Some(first) = targets.first() {
        assert!(first.id.is_some());
    }
}
