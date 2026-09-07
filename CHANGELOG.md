# Changelog

This file records release history for `kalshi-fast-rs`.

Release entries may include a `Compatibility` block summarizing the upstream
Kalshi docs snapshot tracked by that release.

For crate versioning policy and bump rules, see [`VERSIONING.md`](VERSIONING.md).


## [0.8.0] - 2026-09-07

### Compatibility

- Docs snapshot: 2026-09-07
- OpenAPI: 3.29.0
- AsyncAPI: 2.0.0
- Validated through changelog: 2026-09-10

**Changelog entries since 0.7.0 watermark (2026-06-08) and disposition:**

| Entry | Action |
|---|---|
| `available_on_brokers` deprecated (2026-08-27) then removed from event responses (2026-09-10) | **Breaking** — removed `EventData.available_on_brokers` |
| Weather index points expose `receipt_basis` (2026-09-10) | No code change — `GetWeatherIndex` not modeled in crate |
| Margin markets expose `asset_class` (2026-09-10) | No code change — margin markets not modeled |
| Upcoming exchange sharding for commodities/basketball (2026-09-10) | No code change — informational only |
| `center_deci_edge_centi_cent` price level structure emitted again (2026-09-10) | No code change — `price_level_structure` already modeled as a raw passthrough string |
| WebSocket schemas corrected: `seq` on trade/lifecycle/RFQ messages, `order_source` on Margin fill/user-order, `sid`/`seq` on subscription-scoped errors, retired codes 6/16/17, pruned multivariate lifecycle fields, `market_id`/`market_ticker` removed from error schema (2026-09-10) | Fixed `WsError`/`WsErrorRef` to deserialize the wire key `msg` (was incorrectly named `message`, so error text was silently dropped); `sid`/`seq` were already threaded generically for every message type including errors; `order_source` is Margin-only (not modeled); documented retired codes in `docs/spec-parity.md` |
| CF Benchmarks 5Hz value channel `cfbenchmarks_value_5hz` (2026-09-03) | No code change — new channel not yet modeled (tracked gap) |
| Higher FIX market data session limit (2026-09-03) | No code change — FIX not modeled |
| Order identity on FIX market data (2026-09-03) | No code change — FIX not modeled |
| New `GET /margin/fee_tier_rates` (2026-09-03) | No code change — margin endpoint not modeled (tracked gap) |
| Filter FCM orders by `client_order_ids` (2026-09-03) | Added `client_order_ids` to `GetFcmOrdersParams`; `subtrader_id` is now `Option<String>` (**Breaking**, either filter now satisfies the endpoint) |
| Filter historical positions by subaccount (2026-09-03) | Implemented `get_historical_positions` / `GetHistoricalPositionsParams` (endpoint added 2026-07-23, was not previously modeled) with `subaccount` filter |
| Correct remaining counts after crossing order amendments (2026-09-03) | No code change — server-side fix; `remaining_count` already modeled |
| Lower rate-limit cost for cancel-all-orders (2026-09-03) | No code change — operational; cancel-all endpoints not modeled |
| Shard rebalance margin reservation on target balance allocation (2026-09-03) | No code change — `target_balance_allocation` endpoint not modeled (tracked gap) |
| `ClearingBusinessDate` on FIX trade execution reports (2026-09-03) | No code change — FIX not modeled |
| Tapered sub-cent pricing on multivariate markets (`center_deci_edge_centi_cent`) (2026-09-03) | No code change — `price_level_structure` raw string; `price_ranges` already modeled |
| Weather index calibration history endpoint (2026-08-31) | No code change — weather index not modeled (tracked gap) |
| Structured target `details.image_url` (2026-08-29) | No code change — `StructuredTarget.details` is a generic map; new keys surface automatically |
| Localized market content via `Accept-Language` (2026-08-27) | No code change — request header, not a response shape |
| Trade type on FIX market data (2026-08-27) | No code change — FIX not modeled |
| `exchange_index` on `user_orders` WS messages (2026-08-27) | Added `exchange_index` to `WsUserOrder` |
| Cancel-all-orders endpoints (2026-08-27) | No code change — new endpoints not modeled (tracked gap) |
| Historical CF Benchmarks via REST passthrough docs (2026-08-27) | No code change — documentation only |
| Exchange auto-routing enabled by default (2026-08-27) | No code change — behavior only |
| Margin maker-volume incentive programs, `max_reward_per_account` (2026-08-27) | Reconciled `IncentiveProgram`: added `incentive_description` (now required upstream), `max_reward_per_account`; removed stale `target_size` (not in current schema, `target_size_fp` remains) |
| Upcoming exchange sharding: Crypto/Tennis/Baseball (2026-08-24) | No code change — informational only |
| Post-only quotes preserved; crossing rate limits (2026-08-22) | No code change — fee/behavior only |
| Combo RFQ fee assignment for briefly resting orders (2026-08-22) | No code change — fee/behavior only |
| Maker fee exemption for independent NFL combo markets (2026-08-20) | No code change — informational |
| VPC peering for Prime members (2026-08-20) | No code change — informational |
| Entry timestamps for FIX market data (2026-08-20) | No code change — FIX not modeled |
| Cross-shard subaccount transfers (`intra_exchange_instance_transfer`) (2026-08-20) | No code change — new endpoint not modeled (tracked gap) |
| Target balance allocation endpoints (2026-08-20) | No code change — new endpoints not modeled (tracked gap) |
| Resting order value breakdown by exchange index (2026-08-20) | Added `resting_order_value_breakdown: Option<Vec<IndexedBalance>>` to `GetPortfolioRestingOrderTotalValueResponse` |
| Exchange index on portfolio and WebSocket fill records (2026-08-20) | Added `exchange_index` to `MarketPosition`, `Settlement`, `Fill` (REST) and `WsFill` |
| Exchange index filters for portfolio lists (2026-08-20) | Added `exchange_index` filter to `GetOrdersParams`, `GetPositionsParams`, `GetFillsParams` |
| RFQs/combo-market creation for subaccount-restricted keys (2026-08-20) | No code change — permission/behavior only |
| Optional balance reads by `exchange_index` (2026-08-20) | **Breaking** — `get_balance` now takes `GetBalanceParams { subaccount, exchange_index }`; added `balance_breakdown` to `GetBalanceResponse` |
| Exit triggers on margin positions (2026-08-20) | No code change — margin positions not modeled |
| API key location attestation expiry (2026-08-16) | Added `api_key_region_expiration_ts` to `GetApiKeysResponse` |
| New `center_deci_edge_centi_cent` price level structure (2026-08-13) | No code change — raw string passthrough |
| Balance reads scoped by `exchange_index`, `subaccount=0` handling (2026-08-13) | Covered by the `GetBalanceParams` change above |
| Block trade indicator for WebSocket trades (2026-08-13) | Added `is_block_trade` to `WsTrade`/`WsTradeRef` |
| Exchange shard descriptions (2026-08-13) | Added `ExchangeIndexStatus.description` |
| Margin order groups bind to single `exchange_index` (2026-08-13) | No code change — margin order groups not modeled |
| Order group maximum increased to 100,000 (2026-08-13) | No code change — informational limit |
| Richer combo-validation errors on FIX RFQ creation (2026-08-13) | No code change — FIX not modeled |
| Intra-account transfer history endpoints (2026-08-13) | No code change — new endpoints not modeled (tracked gap) |
| Multivariate lookup endpoint and channel removed (2026-08-06) | **Breaking** — removed WS `multivariate` channel, `multivariate`/`multivariate_lookup` message types, and the `WsMultivariate`/`WsMultivariateRef` structs. The deprecated `PUT .../lookup` REST endpoint was never modeled |
| FIX execution reports identify source exchange index (2026-08-06) | No code change — FIX not modeled |
| Sided leverage estimates on margin markets (2026-08-06) | No code change — margin markets not modeled |
| Order group limit updates support subaccounts (2026-08-06) | No code change — `subaccount` param on `PUT order_groups/{id}/limit` not modeled (tracked gap) |
| Multivariate event collections include `exchange_index` (2026-08-06) | No code change — not modeled (tracked gap) |
| `service` field removed from error responses (2026-08-06, deprecated 2026-07-28) | **Breaking** — removed `ErrorResponse.service` |
| Richer combo-validation errors on multivariate market creation (2026-07-30) | No code change — error `message`/`details` already raw strings |
| Lifecycle creation messages include `exchange_index` (2026-07-30) | Added `exchange_index` to `WsMarketLifecycleV2`/`WsEventLifecycle` (shared by `multivariate_market_lifecycle`) |
| Series responses include `exchange_index` (2026-07-30) | Added `Series.exchange_index` |
| New event-keyed live data endpoint (2026-07-30) | No code change — new endpoint not modeled (tracked gap) |
| Subaccount-restricted keys can read order queue positions (2026-07-30) | No code change — permission/behavior only |
| Event `product_metadata` includes `cadence` (2026-07-30) | Added `EventMetadata.cadence` |
| Subaccount-restricted keys can use batch order endpoints (2026-07-30) | No code change — permission/behavior only |
| Subaccount on `quote_created` (2026-07-30) | Added `subaccount` (and `rfq_creator_id`) to `WsQuoteCreated`/`WsQuoteAccepted`/`WsQuoteExecuted` |
| Subaccount-restricted keys can manage order groups (2026-07-30) | No code change — permission/behavior only |
| Order groups limited to 25,000 per user (2026-07-23) | No code change — informational limit |
| Incentive programs on hidden events excluded from listing (2026-07-22/23) | No code change — server-side filtering only |
| Historical positions endpoint (2026-07-23) | Implemented as `get_historical_positions` (see 2026-09-03 above) |
| Subaccount-restricted keys can open WebSocket sessions (2026-07-23) | No code change — permission/behavior only |
| Subaccount-restricted keys can quote on RFQ FIX sessions (2026-07-23) | No code change — FIX not modeled |
| Pyth value WebSocket channel (2026-07-23) | No code change — new channel not modeled (tracked gap) |
| Seven new `price_level_structure` values (2026-07-23) | No code change — raw string passthrough |
| FIX Tag 2446 (`AggressorSide`) on Incremental Refresh (2026-07-09) | No code change — FIX not modeled |
| RFQ-scoped quote lookup endpoint (2026-07-09) | No code change — RFQ-scoped variant not modeled; quote-ID-only lookup retained (tracked gap) |
| Deprecated Predictions REST schema fields removed (2026-07-09) | **Breaking** — removed `Market.response_price_units`, `Market.fractional_trading_enabled`, `MarketPosition.resting_orders_count` (also removed the equivalent WS lifecycle `fractional_trading_enabled` and WS `market_positions`-shared `resting_orders_count`, absent from AsyncAPI too) |
| Margin orders identify system order reasons (2026-07-09) | No code change — margin orders not modeled |
| Exchange announcements endpoint removed (2026-07-04) | **Breaking** — removed `get_exchange_announcements` and `Announcement`/`AnnouncementType`/`AnnouncementStatus`/`GetExchangeAnnouncementsResponse` |
| Multivariate lookup history endpoints fully deprecated (2026-07-02) | No code change — endpoint was never modeled |
| Margin positions `is_portfolio` flag (2026-07-02) | No code change — margin positions not modeled |
| `price_ranges` added to `market_lifecycle_v2` events (2026-07-02) | Added `price_ranges` to `WsMarketLifecycleV2` |
| Per-index exchange status (2026-07-02) | Added `intra_exchange_transfers_active`, `exchange_index_statuses` (`ExchangeIndexStatus`) to `GetExchangeStatusResponse` |
| Per-index subaccount balances (2026-07-02) | Added `exchange_index` to `SubaccountBalance` |
| AcceptQuote FIX reject reason (2026-07-02) | No code change — FIX not modeled |
| More specific FIX cancel/replace rejects (2026-07-02) | No code change — FIX not modeled |
| Sub-account-restricted API keys (2026-07-02) | Added `subaccount`/`fcm_subtrader_id` to `ApiKey`, `CreateApiKeyRequest`, `GenerateApiKeyRequest`; added `warning` to `CreateApiKeyResponse` |
| Trade-scoped API key permissions `write::trade` (2026-06-30) | No code change — scopes already `Vec<String>` |
| Margin positions `margin_used` omitted for jointly-margined positions (2026-06-29) | No code change — margin positions not modeled |
| Margin risk per-market metrics limited (2026-06-26) | No code change — margin risk not modeled |
| RFQ quote retention and RFQ-scoped quote actions (2026-06-25) | No code change — RFQ-scoped action-endpoint variants not modeled; quote-ID-only actions retained. Retention nuance documented in `docs/spec-parity.md` (tracked gap) |
| API usage tier qualification requirements halved (2026-06-25) | No code change — informational |
| FIX exchange index routing (2026-06-25) | No code change — FIX not modeled |
| RFQ quotes support post-only on FIX (2026-06-24) | No code change — FIX not modeled |
| Get Quote rate-limit cost reduced (2026-06-23) | No code change — crate doesn't model per-endpoint token costs |
| RFQ quote market/event filters removed (2026-06-20) | **Breaking** — removed `market_ticker`/`event_ticker` from `GetQuotesParams`; added `min_ts`/`max_ts`/`user_filter` |
| Communications retention window reduced to 7 days (2026-06-19) | No code change — informational |
| `settlement_sources` added to events API (2026-06-18) | Added `EventData.settlement_sources` |
| `strike_type`/`cap_strike` on `metadata_updated` (2026-06-18) | Added `strike_type`, `cap_strike`, `custom_strike` top-level to `WsMarketLifecycleV2` (alongside existing `floor_strike`/`yes_sub_title`) |
| RFQ quote identity on FIX (2026-06-18) | No code change — FIX not modeled |
| Trade entries in FIX market data (2026-06-18) | No code change — FIX not modeled |
| Legacy order mutation endpoints deprecated (2026-06-18/25) | Marked `create_order`, `cancel_order`, `amend_order`, `decrease_order`, `batch_create_orders`, `batch_cancel_orders` `#[deprecated]` pointing at the `*_v2` equivalents; migrated `examples/place_order.rs` and `tests/rest_orders.rs` to the V2 endpoints |
| Event tickers filter on `GET /events` (2026-06-18) | Added `tickers` to `GetEventsParams` |
| Subaccount on margin positions (2026-06-18) | No code change — margin positions not modeled |
| Block-trade accept API key permissions (2026-06-18) | No code change — scopes already `Vec<String>` |
| Sanity limits enforced on orderbook subscriptions (2026-06-18) | No code change — informational limits |
| Quote time filters and pagination fix (2026-06-18) | Added `min_ts`/`max_ts` to `GetQuotesParams` (see 2026-06-20 above); pagination fix is server-side only |
| API usage volume progress endpoint (2026-06-11) | No code change — new endpoint not modeled (tracked gap) |
| Perps mark prices on margin markets (2026-06-11) | No code change — margin markets not modeled |
| Self-serve Advanced API usage tier upgrade (2026-06-11) | No code change — new endpoint not modeled (tracked gap) |
| Margin fee-tier endpoint returns active rates (2026-06-11) | No code change — already handled in 0.6.0 |
| Perps volume/open-interest notional fields (2026-06-11) | No code change — margin markets/`margin_ticker` not modeled |
| Tick size added to `GET Margin Markets` (2026-06-11) | No code change — margin markets not modeled |
| Fractional quantities for RFQs (2026-06-11) | No code change — `contracts_fp` already present (handled in 0.6.0) |
| `FeeType::quadratic_with_combo_maker_fees` (present in current OpenAPI enum, not called out in a dedicated changelog entry) | Added `FeeType::QuadraticWithComboMakerFees` |

### Breaking

- [Rust API] Removed `Market.response_price_units`, `Market.fractional_trading_enabled`, `MarketPosition.resting_orders_count`, `Event.available_on_brokers`, and the WS lifecycle `fractional_trading_enabled` field. None of these appear in the current OpenAPI/AsyncAPI schemas.
- [Rust API] Removed the WebSocket `multivariate` channel (`WsChannelV2::Multivariate`), the `multivariate`/`multivariate_lookup` message types, and the `WsMultivariate`/`WsMultivariateRef`/`WsMultivariateSelectedMarket(Ref)` structs. The channel no longer exists upstream; subscriptions now return an unknown-channel error. Use `multivariate_market_lifecycle` for multivariate market state changes.
- [Rust API] Removed `get_exchange_announcements`, `GetExchangeAnnouncementsResponse`, `Announcement`, `AnnouncementType`, `AnnouncementStatus`. `GET /exchange/announcements` was removed from the Predictions REST API.
- [Rust API] Removed `ErrorResponse.service`. The field was removed from all REST error bodies; branch on `code` instead.
- [Rust API] Removed `market_ticker`/`event_ticker` from `GetQuotesParams`. `GET /communications/quotes` no longer supports these filters; filter by RFQ, status, user, or update time instead.
- [Rust API] `get_balance` now takes a `GetBalanceParams` argument (`subaccount`, `exchange_index`) instead of no arguments.
- [Rust API] `GetFcmOrdersParams.subtrader_id` is now `Option<String>` (was `String`); at least one of `subtrader_id` or the new `client_order_ids` is required by the exchange.
- [Rust API] `IncentiveProgram.target_size` was removed (not present in the current schema; `target_size_fp` remains). `incentive_description` is now a required, non-`Option` field.
- Deprecated (not removed): `create_order`, `cancel_order`, `amend_order`, `decrease_order`, `batch_create_orders`, `batch_cancel_orders` are marked `#[deprecated]`. The exchange now rejects live calls to these legacy `/portfolio/orders` mutation endpoints with a message to switch to the V2 event-order endpoints (`*_v2`); the Rust methods are retained for source compatibility but should not be used for new code.

### Added

- [Rust API] `get_historical_positions` / `GetHistoricalPositionsParams` for `GET /historical/positions` (added upstream 2026-07-23; `subaccount` filter added 2026-09-03).
- [Rust API] `exchange_index` surfaced across the many REST/WS types upstream added it to: `MarketPosition`, `Settlement`, `Fill`, `Series`, `EventData`, `GetPositionsParams`, `GetOrdersParams`, `GetFillsParams`, `WsFill`, `WsUserOrder`, `WsMarketLifecycleV2` (created events), `WsEventLifecycle`.
- [Rust API] `GetExchangeStatusResponse.intra_exchange_transfers_active` and `.exchange_index_statuses: Vec<ExchangeIndexStatus>`.
- [Rust API] `GetPortfolioRestingOrderTotalValueResponse.resting_order_value_breakdown` and `GetBalanceResponse.balance_breakdown`, both `Vec<IndexedBalance>`.
- [Rust API] `is_block_trade` on `WsTrade`/`WsTradeRef`.
- [Rust API] `api_key_region_expiration_ts` on `GetApiKeysResponse`; `subaccount`/`fcm_subtrader_id` on `ApiKey`, `CreateApiKeyRequest`, `GenerateApiKeyRequest`; `warning` on `CreateApiKeyResponse`.
- [Rust API] `settlement_sources` and `exchange_index` on `EventData`; `cadence` on `EventMetadata`.
- [Rust API] `strike_type`, `cap_strike`, `custom_strike`, `price_ranges` top-level on `WsMarketLifecycleV2` (present on `metadata_updated` / `created` / `price_level_structure_updated` events respectively).
- [Rust API] `subaccount` and `rfq_creator_id` on `WsQuoteCreated`/`WsQuoteAccepted`; `subaccount` on `WsQuoteExecuted`.
- [Rust API] `tickers` filter on `GetEventsParams`; `min_ts`/`max_ts`/`user_filter` on `GetQuotesParams`; `client_order_ids` on `GetFcmOrdersParams`.
- [Rust API] `FeeType::QuadraticWithComboMakerFees`.
- [Rust API] `IncentiveProgram.max_reward_per_account` (margin maker-volume programs).

### Fixed

- [Rust API] `WsError`/`WsErrorRef` now correctly deserialize the error text from the wire key `msg` (the field was previously mapped from a non-existent `message` key, so `WsError::message` was always `None`). Confirmed against the AsyncAPI `errorResponsePayload` schema.

### Docs

- [Docs] Documented the retired WS error codes (6, 16, 17) and the several dispositioned "not yet modeled" upstream surfaces (margin markets/positions/risk/order-groups, FIX, several new REST endpoints and WS channels) in `docs/spec-parity.md`.


## [0.7.0] - 2026-08-12

### Compatibility

- Docs snapshot: 2026-06-08
- OpenAPI: 3.20.0
- AsyncAPI: 2.0.0
- Validated through changelog: 2026-06-08

### Fixed

- [Rust API] Preserved Kalshi WebSocket subscription cursor metadata on unknown frames in both
  owned and borrowed parsing paths. `WsMessageV2::subscription_id()` / `.sequence()` and
  `WsMessageRef::subscription_id()` / `.sequence()` now return unknown-frame `sid` / `seq`.

### Breaking

- [Rust API] All public `WsMessageV2` and `WsMessageRef` control and `Unknown` variants carry
  `sid` and `seq`. Downstream constructors and exhaustive matches must supply or handle the new
  fields (or use `..`). Consumers accounting for the per-subscription cursor MUST use
  `subscription_id()` and `sequence()` rather than matching message variants.


## [0.6.0] - 2026-06-08

### Compatibility

- Docs snapshot: 2026-06-08
- OpenAPI: 3.20.0
- AsyncAPI: 2.0.0
- Validated through changelog: 2026-06-08

**Changelog entries since 0.5.0 watermark (2026-06-04) and disposition:**

| Entry | Action |
|---|---|
| Margin fee-tier returns active rates (2026-06-03/11) | No code change — exchange bug fix only |
| Perps volume/OI notional fields on margin markets (2026-06-05/11) | No code change — margin market types not in crate |
| Tick size on `GET /margin/markets` (2026-06-03/11) | No code change — margin market types not in crate |
| Automated API rate-limit tiers / grants (2026-06-06) | **Breaking** — replaced `GetAccountApiLimitsResponse`; added `BucketLimit`, `ApiUsageLevelGrant`; added `GET /account/endpoint_costs` (`get_account_endpoint_costs`, `GetAccountEndpointCostsResponse`, `EndpointTokenCost`) |
| Fractional contract quantities for RFQs (2026-05-26/2026-06-11) | No code change — `contracts_fp` already present in `CreateRfqRequest` |
| Legacy order endpoints cost 10× rate-limit tokens (2026-06-04) | No code change — operational rate-limit change only |
| Post Only Cross Cancel `last_update_reason` value (2026-06-04) | No code change — `last_update_reason` not modeled in `Order`; tolerated by existing `extra` flatten if present |
| Transfer-scoped API key permissions (2026-06-03) | No code change — scopes stored as `Vec<String>` already |
| Block trade indicators on public trade endpoints (2026-05-29/2026-06-01) | Added `is_block_trade` to `Trade` and `GetTradesParams` |
| V2 event-order endpoints (`/portfolio/events/orders/*`) | Added all V2 types and six new `KalshiRestClient` methods |
| `cfbenchmarks_value` AsyncAPI channel | Added full channel, subscription, and message support |
| `FeeType::quadratic_with_maker_fees` | Added `QuadraticWithMakerFees` variant to `FeeType` enum |

### Added

- [Rust API] Added `is_block_trade: bool` (with `#[serde(default)]`) to the public REST `Trade`
  struct (2026-05-29). Defaults to `false` for payloads predating the flag.
- [Rust API] Added `is_block_trade: Option<bool>` filter to `GetTradesParams` so callers can filter
  by block-trade status on `GET /markets/trades` and `GET /historical/trades`.
- [Rust API] Added all V2 event-order types and six new `KalshiRestClient` methods for the lower-cost
  `/portfolio/events/orders/*` endpoints: `create_order_v2`, `cancel_order_v2`, `amend_order_v2`,
  `decrease_order_v2`, `batch_create_orders_v2`, `batch_cancel_orders_v2`. These endpoints use a
  single price + `BookSide` instead of separate yes/no prices.
  New request/response types: `CreateOrderV2Request`, `CreateOrderV2Response`,
  `CancelOrderV2Params`, `CancelOrderV2Response`, `AmendOrderV2Request`, `AmendOrderV2Response`,
  `DecreaseOrderV2Request`, `DecreaseOrderV2Response`, `BatchCreateOrdersV2Request`,
  `BatchCreateOrderV2OrderResponse`, `BatchCreateOrdersV2Response`,
  `BatchCancelOrderV2RequestOrder`, `BatchCancelOrdersV2Request`,
  `BatchCancelOrderV2OrderResponse`, `BatchCancelOrdersV2Response`.
- [Rust API] Added `BucketLimit` and `ApiUsageLevelGrant` structs (2026-06-06). `BucketLimit` holds
  `refill_rate: i64` and `bucket_capacity: i64`. `ApiUsageLevelGrant` holds `exchange_instance`,
  `level`, `source: String`, and `expires_ts: Option<i64>` (absent for non-expiring grants).
- [Rust API] Added `get_account_endpoint_costs()` method and `GetAccountEndpointCostsResponse` /
  `EndpointTokenCost` structs for the new public `GET /account/endpoint_costs` endpoint, which lists
  API v2 endpoints whose token cost differs from the default cost.
- [Rust API] Added CF Benchmarks subscription-update support so the documented post-subscribe
  workflow is reachable: `WsUpdateAction::SubscribeIndices` / `UnsubscribeIndices` / `Indexlist`
  variants and an `index_ids: Option<Vec<String>>` field on `WsUpdateSubscriptionParamsV2`. The
  subscription tracker now folds index add/remove updates into the resubscribe state, and
  `validate_update` enforces that index actions carry no market targets and that
  `subscribe_indices` / `unsubscribe_indices` include `index_ids`.
- [Rust API] Added `FeeType::QuadraticWithMakerFees` variant (serialized
  `quadratic_with_maker_fees`). `FeeType` now also carries an `#[serde(other)] Unknown` catch-all
  so unknown future variants never panic.
- [Rust API] Added full `cfbenchmarks_value` channel support:
  - `WsChannelV2::CfbenchmarksValue` variant
  - `index_ids: Option<Vec<String>>` parameter on `WsSubscriptionParamsV2` (use `["all"]` for all
    indices)
  - `WsMsgType::CfbenchmarksValue` and `WsMsgType::CfbenchmarksValueIndexlist` variants
  - New types `WsCfBenchmarksValue`, `WsCfBenchmarksValueRef`, `WsCfBenchmarksAvgData`,
    `WsCfBenchmarksIndexList`, `WsCfBenchmarksIndexListRef` in `ws::types::messages::cfbenchmarks`
  - `WsDataMessageV2::CfbenchmarksValue` and `WsDataMessageV2::CfbenchmarksValueIndexlist` variants
    routed through both the wire and envelope parse paths


### Changed

- [Rust API] `GetAccountApiLimitsResponse` now reflects the current OpenAPI shape: nested
  `read: BucketLimit` and `write: BucketLimit` objects plus `grants: Vec<ApiUsageLevelGrant>`.
  The old flat `read_limit: i64` / `write_limit: i64` fields are removed.

### Breaking

- [Rust API] `GetAccountApiLimitsResponse` field layout changed (automated API rate-limit tiers,
  2026-06-06). Replace `resp.read_limit` → `resp.read.refill_rate` (or `.bucket_capacity`) and
  `resp.write_limit` → `resp.write.refill_rate`. The `grants` field is new; downstream exhaustive
  struct destructuring must add it.
- [Rust API] `WsUpdateAction` gained `SubscribeIndices`, `UnsubscribeIndices`, and `Indexlist`
  variants, and `WsUpdateSubscriptionParamsV2` gained an `index_ids` field. Downstream code with
  exhaustive matches over `WsUpdateAction` or struct-literal construction of
  `WsUpdateSubscriptionParamsV2` must be updated.



## [0.5.0] - 2026-05-29

### Compatibility

- Docs snapshot: 2026-05-29
- Validated through changelog: 2026-06-04

### Added

- [Rust API] Added `BookSide` enum (`Bid` | `Ask` | `Unknown`) to `types.rs` for the normalized
  `book_side` field added to order/fill responses on 2026-05-07.
- [Rust API] Added `outcome_side: Option<YesNo>` and `book_side: Option<BookSide>` fields to
  `Order`, `Fill`, `WsFill`, `WsFillRef`, and `WsUserOrder`. These are the normalized direction
  fields Kalshi added on 2026-05-07 (`bid` ≡ `yes`, `ask` ≡ `no`).
- [Rust API] Added `taker_outcome_side: Option<TradeTakerSide>` and `taker_book_side:
  Option<BookSide>` to the public `Trade` (REST) and `WsTrade` / `WsTradeRef` (WebSocket) objects,
  matching the normalized taker-direction fields added to trade responses on 2026-05-07.
- [Rust API] Added `balance_dollars: Option<FixedPointDollars>` to `GetBalanceResponse` for the
  centi-cent precision balance field added on 2026-05-28 (direct members only).
- [Rust API] Added `subaccount: Option<u32>` to `CreateOrderGroupResponse` for the field added on
  2026-05-07 (0 = primary, 1–32 = subaccount).
- [Rust API] Added `rfq_user_filter: Option<String>` to `GetQuotesParams` for the filter parameter
  added on 2026-05-07. Pass `"self"` to restrict to quotes on the authenticated user's RFQs.
- [Rust API] Added `WsMarketLifecycleEventType::MetadataUpdated` variant for the new lifecycle event
  type added on 2026-05-11, fired when market metadata (name, title, subtitles) changes.
- [Rust API] Surfaced the top-level `metadata_updated` payload values on `WsMarketLifecycleV2` /
  `WsMarketLifecycleV2Ref`: added `floor_strike: Option<f64>` and `yes_sub_title: Option<String>`
  (per AsyncAPI these appear at the top level only on `metadata_updated`, distinct from the
  `additional_metadata.*` copies emitted on creation), plus a top-level flatten `extra` map so other
  conditional lifecycle keys are no longer silently discarded.
- [Rust API] Added the `event_fee_update` WebSocket message: new `WsEventFeeUpdate` /
  `WsEventFeeUpdateRef` types, a `WsMsgType::EventFeeUpdate` variant, and
  `WsDataMessageV2::EventFeeUpdate` / `WsDataMessageRef::EventFeeUpdate` variants. This message is
  delivered on the existing `market_lifecycle_v2` channel and carries `event_ticker`,
  `fee_type_override`, and `fee_multiplier_override` (both overrides `null` when cleared).
  Previously these messages surfaced as `WsMessageV2::Unknown`.
- [Rust API] Added the spec-required `ts_ms` (matching-engine timestamp, ms) to `WsOrderGroupUpdate`
  and `WsOrderGroupUpdateRef`, which were previously dropping the field.
- [Rust API] Added `get_margin_fee_tiers()` method and `GetMarginFeeTiersResponse` struct for the
  `GET /margin/fee_tiers` endpoint. The response uses `maker_fee_rates` / `taker_fee_rates` (market
  ticker → decimal fee rate maps, fee = `notional * rate`).
- [Tests] Added `ws_fill_normalized_fields_parse` test covering the new `outcome_side` / `book_side`
  fields on `WsFill`.

### Changed

- [Rust API] Updated `KalshiEnvironment::demo()` and `KalshiEnvironment::production()` to use the
  dedicated external API hosts introduced on 2026-05-07. REST hosts: `external-api.demo.kalshi.co` /
  `external-api.kalshi.com`. WS hosts: `external-api-ws.demo.kalshi.co` /
  `external-api-ws.kalshi.com`. The old hosts (`demo-api.kalshi.co`, `api.elections.kalshi.com`)
  are no longer used.

### Breaking

- [Rust API] `Order.side` changed from `YesNo` to `Option<YesNo>`. The `side` field was deprecated
  by Kalshi on 2026-05-07 and removed ~2026-05-28. Downstream code must use `outcome_side` (or
  handle `None`).
- [Rust API] `Order.action` changed from `BuySell` to `Option<BuySell>`. Same deprecation/removal
  timeline as `Order.side`. Use `book_side` instead.
- [Rust API] `Fill.side` changed from `YesNo` to `Option<YesNo>` for the same reason.
- [Rust API] `Fill.action` changed from `BuySell` to `Option<BuySell>` for the same reason.
- [Rust API] `WsFill.side` changed from `YesNo` to `Option<YesNo>` for the same reason.
- [Rust API] `WsFill.action` changed from `BuySell` to `Option<BuySell>` for the same reason.
- [Rust API] `Trade.taker_side` and `WsTrade.taker_side` changed from `TradeTakerSide` to
  `Option<TradeTakerSide>`. The `taker_side` field was deprecated on 2026-05-07 in favor of
  `taker_outcome_side` / `taker_book_side`. Downstream code must handle `None`.
- [Rust API] `KalshiEnvironment::demo()` and `KalshiEnvironment::production()` now point to the new
  dedicated external API hostnames. Code that hard-coded the old host strings must update.
- [Upstream] `GET /margin/fee_tiers` response no longer returns `maker_fee_tiers` /
  `taker_fee_tiers` tier-name maps; it now returns `maker_fee_rates` / `taker_fee_rates` decimal
  maps. `GetMarginFeeTiersResponse` was added with the new shape (no old shape existed in this
  crate).


## [0.4.0] - 2026-04-18

### Compatibility

- Docs snapshot: 2026-04-18
- OpenAPI: 3.13.0
- AsyncAPI: 2.0.0
- Validated through changelog: 2026-04-16

### Added

- [Rust API] Added REST helpers for current Kalshi endpoints and aliases, including `get_market_orderbooks`, `get_trades_historical`, `get_fills_historical`, `get_live_data_by_milestone`, `get_game_stats`, and `get_market_candlesticks_historical`.
- [Rust API] Added current OpenAPI fields used by the refreshed docs, including `occurrence_datetime` on event and market payloads, `series_ticker` on historical market filters, and fixed-point quote contract fields.
- [Docs] Added `VERSIONING.md` plus repo guidance that points refresh work at the live Kalshi docs, changelog RSS, OpenAPI, and AsyncAPI documents instead of checked-in spec snapshots.

### Changed

- [Rust API] Restored `GetOrderQueuePositionsParams` to the current OpenAPI behavior by allowing unfiltered queue-position requests.
- [Rust API] Migrated the WebSocket public surface to the current V2 contract, including `WsChannelV2`, `WsMessageV2`, `WsDataMessageV2`, `WsSubscriptionParamsV2`, and the `subscribe_v2` / `unsubscribe_v2` / `update_subscription_v2` / `start_reader_v2` / `next_event_v2` methods.
- [Rust API] Aligned authenticated REST response structs with the current OpenAPI fixed-point contract for `Order`, `Trade`, `Fill`, `Settlement`, `MarketPosition`, and `EventPosition`.
- [Rust API] Aligned communications REST and WebSocket quote/RFQ payloads with the current fixed-point-only docs by removing stale integer compatibility fields and relying on `*_dollars` and `*_fp` fields.
- [Upstream] Validated the current Kalshi docs snapshot against the changelog items covering historical `series_ticker` filtering, fixed-point response cleanup, millisecond WebSocket timestamps, and `occurrence_datetime` on market responses.
- [Tests] Refreshed parsing fixtures to the current OpenAPI/AsyncAPI field sets, added coverage for `occurrence_datetime`, and added deterministic V2 WebSocket command-behavior coverage.
- [Tests] Updated live integration coverage to use the filters and account-scope assumptions required by the current communications, queue-position, and FCM-only portfolio endpoints.
- [Upstream] Updated docs, examples, and tests for Kalshi's current WebSocket handshake behavior, which now requires authenticated connections even when subscribing only to public channels.
- [Docs] Tightened the refresh workflow to remove upstream-removed schema fields and response shapes from the public Rust API instead of preserving compatibility shims by default.

### Removed

- [Docs] Removed vendored OpenAPI/AsyncAPI snapshots, spec manifest artifacts, the parity generation script, and raw spec contract tests in favor of live upstream docs plus concise `docs/spec-parity.md` notes.
- [Rust API] Removed stale REST compatibility fields and aliases that are no longer present in the current OpenAPI, including legacy fill/settlement fixed-point aliases.
- [Rust API] Removed stale WebSocket fill aliases for `yes_price_fixed` and `no_price_fixed` so parsing follows the current AsyncAPI names.
- [Rust API] Removed stale quote and RFQ integer compatibility fields from REST and WebSocket communications payloads.
- [Rust API] Removed stale WebSocket compatibility fields and shapes from `WsTicker`, `WsTrade`, `WsOrderbookSnapshot`, `WsOrderbookDelta`, and `WsFill`; downstream consumers must use the current `*_dollars` and `*_fp` fields from the live AsyncAPI contract.
- [Rust API] Removed the stale `GetMarketOrderbookResponse.orderbook` compatibility view and its synthesized integer orderbook shape; the current OpenAPI response is `orderbook_fp` only.

### Breaking

- [Rust API] Downstream WebSocket code must migrate from the pre-V2 types and methods such as `WsChannel`, `WsMessage`, `WsDataMessage`, `subscribe`, `unsubscribe`, `update_subscription`, `start_reader`, and `next_event` to the V2 names and `*_v2` methods.
- [Rust API] `KalshiWsClient::connect` and `KalshiWsLowLevelClient::connect` no longer provide an unauthenticated public-channel path; downstream code must use `connect_authenticated`, even for public subscriptions.
- [Rust API] V2 subscription validation is stricter: `orderbook_delta` requires `market_ticker` or `market_tickers`, rejects `market_id` and `market_ids`, and enforces exclusive market-target fields on subscribe and update commands.
- [Rust API] Downstream code must update authenticated REST response field access to the current spec names such as `fill_count_fp`, `remaining_count_fp`, `initial_count_fp`, `last_update_time`, `subaccount_number`, `total_traded_dollars`, `market_exposure_dollars`, `total_cost_dollars`, and `total_cost_shares_fp`.
- [Rust API] Legacy integer/count response fields and compatibility aliases previously accepted by `Order`, `Trade`, `Fill`, `Settlement`, `MarketPosition`, and `EventPosition` are no longer exposed by the public Rust types.
- [Rust API] Downstream WebSocket code can no longer access removed compatibility fields such as `price`, `yes_bid`, `yes_ask`, `volume`, `open_interest`, `count`, `yes_price`, `no_price`, `delta`, `no_price_dollars`, or the legacy integer orderbook snapshot levels on current V2 message types.
- [Rust API] Downstream REST code must read `GetMarketOrderbookResponse.orderbook_fp` directly; the legacy `orderbook` field has been removed.

## [0.3.0] - 2026-03-05

### Compatibility

- Not recorded for this historical release.

### Added

- [Rust API] Added `MarketStatusConversionError` for strict lifecycle/query status conversions.
- [Rust API] Added best-effort `From` conversions between lifecycle `MarketStatus` and query `MarketStatusQuery`.
- [Rust API] Added strict `TryFrom<&...>` conversions for exact one-to-one status mapping.
- [Tests] Added and expanded parsing tests for status serialization and conversion behavior.
- [Rust API] Added `KalshiError::Parse` with parse context, human-readable reason, raw payload bytes, and optional serde source error.
- [Rust API] Added public parse accessors on `KalshiError`: `parse_context()`, `parse_error_reason()`, and `parse_raw_bytes()`.
- [Tests] Added regression tests covering REST and WebSocket parse failures to verify reason text and raw-byte preservation.

### Changed

- [Rust API] Renamed query enum `MarketStatus` to `MarketStatusQuery`.
- [Rust API] Renamed REST market lifecycle enum `MarketState` to `MarketStatus`.
- [Rust API] Updated `GetMarketsParams.status` to use `Option<MarketStatusQuery>`.
- [Rust API] Updated `Market.status` to use `Option<MarketStatus>`.
- [Docs] Updated examples, tests, and REST module docs to use the new names.
- [Rust API] REST success-response decoding now returns `KalshiError::Parse` with raw bytes instead of a plain serde JSON error.
- [Rust API] WebSocket envelope and message parsing now returns `KalshiError::Parse` with clearer parse-failure context and preserved raw payload bytes.

### Removed

- [Rust API] Removed old `MarketState` and old query `MarketStatus` names without aliases.

### Breaking

- [Rust API] Downstream consumers must update imports and enum references to the new names.
- [Rust API] Downstream exhaustive `match` statements over `KalshiError` must handle the new `Parse` variant.
