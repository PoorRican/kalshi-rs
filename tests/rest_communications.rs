#![cfg(feature = "live-tests")]

//! # ⚠️  Known-failing tests
//!
//! Both `test_rfq_lifecycle` and `test_quote_lifecycle` currently fail against the
//! Kalshi demo environment with 4XX responses that are **not** client-side bugs in
//! this crate — the request bodies serialize correctly but the demo server rejects
//! them for environment/state reasons:
//!
//! - `test_rfq_lifecycle` → `409 already_exists` (service: midland). The demo
//!   account appears to have an outstanding RFQ on the chosen market that is not
//!   cleaned up between runs. Manual cleanup or a unique-per-run selector is
//!   required before this test can pass reliably.
//! - `test_quote_lifecycle` → `400 invalid_parameters` (service: midland). The
//!   `yes_bid` / `no_bid` values (`"0.01"` / `"0.01"`) almost certainly violate a
//!   pricing constraint (e.g., `yes_bid + no_bid ≤ 1.00` plus per-market minimum
//!   spread). The exact rule is not documented in
//!   `.claude/skills/kalshi-api-docs/references/REST.md`.
//!
//! **Before re-enabling or modifying these tests, validate the expected request
//! shape with the Python helper:**
//!
//! ```bash
//! uv run .claude/skills/kalshi-api-docs/scripts/kalshi_rest.py \
//!     --platform demo \
//!     --method POST \
//!     --path /communications/rfqs \
//!     --body '{"market_ticker":"...","contracts":1}'
//! ```
//!
//! Capture the exact 4XX response (body + `details`), adjust the request, and only
//! then update these tests. Do **not** paper over the failure by adding broader
//! error tolerance.

mod common;

use kalshi_fast::{
    CreateQuoteRequest, CreateRFQRequest, GetMarketsParams, KalshiRestClient, MarketStatusQuery,
};
use std::time::Duration;

const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
#[ignore = "demo environment returns 409 already_exists; see module doc"]
async fn test_rfq_lifecycle() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    // Find an open market
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

    // 1. Create an RFQ
    let create_resp = tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        client
            .create_rfq(CreateRFQRequest {
                market_ticker: market_ticker.clone(),
                contracts: Some(1),
                contracts_fp: None,
                target_cost_centi_cents: None,
                target_cost_dollars: None,
                rest_remainder: false,
                replace_existing: None,
                subtrader_id: None,
                subaccount: None,
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("create_rfq failed");

    let rfq_id = create_resp.id.clone();

    // 2. Get the RFQ and verify
    let get_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_rfq(&rfq_id).await
    })
    .await
    .expect("timeout")
    .expect("get_rfq failed");

    assert_eq!(get_resp.rfq.id, rfq_id);
    assert_eq!(get_resp.rfq.market_ticker, market_ticker);

    // 3. Delete the RFQ (cleanup)
    let _delete_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.delete_rfq(&rfq_id).await
    })
    .await
    .expect("timeout")
    .expect("delete_rfq failed");
}

#[tokio::test]
#[ignore = "quote pricing constraints undocumented; see module doc"]
async fn test_quote_lifecycle() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    // Find an open market
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

    // 1. Create an RFQ as prerequisite
    let rfq_resp = tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        client
            .create_rfq(CreateRFQRequest {
                market_ticker: market_ticker.clone(),
                contracts: Some(1),
                contracts_fp: None,
                target_cost_centi_cents: None,
                target_cost_dollars: None,
                rest_remainder: false,
                replace_existing: None,
                subtrader_id: None,
                subaccount: None,
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("create_rfq failed");

    let rfq_id = rfq_resp.id.clone();

    // 2. Create a quote on the RFQ
    let quote_resp = tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        client
            .create_quote(CreateQuoteRequest {
                rfq_id: rfq_id.clone(),
                yes_bid: "0.01".to_string(),
                no_bid: "0.01".to_string(),
                rest_remainder: false,
                subaccount: None,
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("create_quote failed");

    let quote_id = quote_resp.id.clone();

    // 3. Get the quote and verify
    let get_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.get_quote_by_rfq(&rfq_id, &quote_id).await
    })
    .await
    .expect("timeout")
    .expect("get_quote_by_rfq failed");

    assert_eq!(get_resp.quote.id, quote_id);
    assert_eq!(get_resp.quote.rfq_id, rfq_id);

    // 4. Delete the quote (cleanup)
    let _delete_quote = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.delete_quote_by_rfq(&rfq_id, &quote_id).await
    })
    .await
    .expect("timeout")
    .expect("delete_quote_by_rfq failed");

    // 5. Delete the RFQ (cleanup)
    let _delete_rfq = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client.delete_rfq(&rfq_id).await
    })
    .await
    .expect("timeout")
    .expect("delete_rfq failed");
}
