# Changelog

This file records release history for `kalshi-fast-rs`.

Release entries may include a `Compatibility` block summarizing the upstream
Kalshi docs snapshot tracked by that release.

For crate versioning policy and bump rules, see [`VERSIONING.md`](VERSIONING.md).


## [0.8.0] - 2026-09-08

### Compatibility

- Docs snapshot: 2026-09-08
- OpenAPI: 3.29.0
- AsyncAPI: 2.0.0
- Validated through changelog: 2026-09-10

Kalshi's public changelog page (`/changelog/index.md`) publishes some entries a few
days ahead of their effective date (e.g. exchange-sharding notices, the
`available_on_brokers` removal). The live OpenAPI/AsyncAPI specs already reflect
the post-change shape as of this snapshot, so this release follows the spec
(source of truth for shape) rather than waiting for each entry's effective date.

**Changelog entries since the 0.7.0 watermark (2026-06-08) and disposition:**

113 changelog entries were published between 2026-06-11 and 2026-09-10. The large
majority are either FIX-API-only (this crate has no FIX support), Margin-exchange
trading surfaces (positions/orders/risk/leverage — not modeled; only the
already-tracked `/margin/fee_tiers` fee-rate endpoint is), FCM-subtrader-only, or
operational/permission/rate-limit notices with no schema impact. Those are grouped
below. Entries with real Rust-API impact are listed individually.

| Entry (date) | Action |
|---|---|
| `available_on_brokers` removed from event responses (2026-08-27 deprecated / 2026-09-10 removed) | **Breaking** — removed `EventData.available_on_brokers` |
| Deprecated REST schema fields removed: `Market.response_price_units`, `Market.fractional_trading_enabled`, `MarketPosition.resting_orders_count` (2026-07-09) | **Breaking** — removed all three fields |
| `GET /exchange/announcements` removed (2026-07-04) | **Breaking** — removed `get_exchange_announcements`, `GetExchangeAnnouncementsResponse`, `Announcement`, `AnnouncementType`, `AnnouncementStatus` |
| Multivariate lookup endpoint and channel removed (2026-08-06); lookup history fully deprecated (2026-07-02) | **Breaking** — removed `lookup_tickers_for_market_in_multivariate_event_collection` (PUT lookup), `get_multivariate_event_collection_lookup_history` (GET lookup history) and their request/response types; removed the WebSocket `multivariate` channel (`WsChannelV2::Multivariate`, `WsMsgType::Multivariate`/`MultivariateLookup`, `WsMultivariate`/`WsMultivariateRef`) |
| RFQ quote `market_ticker`/`event_ticker` filters removed from `GET /communications/quotes` (2026-06-20) | **Breaking** — removed `GetQuotesParams.market_ticker`/`.event_ticker`; added `.user_filter` |
| `center_deci_edge_centi_cent` price-level structure emitted again; 7 new `price_level_structure` values (2026-08-13, 2026-07-23) | No code change — `price_level_structure` is modeled as an opaque `String` per upstream guidance to read `price_ranges` dynamically rather than hardcode structure names |
| WebSocket schemas corrected: `seq` documented on trade/lifecycle/multivariate-lifecycle/event-lifecycle/event-fee-update/RFQ messages; `sid`/`seq` on subscription-scoped errors; error codes 6/16/17 retired; `market_id`/`market_ticker` removed from the error schema (2026-09-10) | No code change — the envelope already surfaces `sid`/`seq` generically on every message including `Error`, and `WsError` never modeled `market_id`/`market_ticker` |
| `GET /account/limits`, deprecated `service` error field removed (2026-08-06) | No code change — `ErrorResponse.service` was already `Option<String>` |
| Richer combo-validation error bodies on multivariate creation/RFQ (2026-07-30, 2026-08-13) | No code change — `ErrorResponse.message`/`.details` already carry the richer text; `code` values are unchanged |
| Structured target `details.image_url` (2026-08-29) | No code change — `StructuredTarget.details` is already a generic `Map<String, Value>` |
| `GET /api_keys` returns `api_key_region_expiration_ts` (2026-08-16) | No code change — `ApiKey` has a flatten `extra` catch-all |
| **Exchange sharding rollout** — `exchange_index` added across many REST/WS surfaces (2026-06-18 through 2026-09-10) | **Breaking** where the field is spec-required — added `exchange_index` to `Fill`, `Settlement`, `MarketPosition` (`i64`, required), `SubaccountBalance` (`i64`, required), `Market`, `Series`, `MultivariateEventCollection` (`Option<i64>`); added `exchange_index` filter to `GetPositionsParams`/`GetFillsParams`/`GetOrdersParams`; added `GetBalanceParams`/`get_balance_scoped`; added `exchange_index` to `ApplySubaccountTransferRequest`, `IntraExchangeInstanceTransferRequest`; added `WsMarketLifecycleV2.exchange_index`, `WsEventLifecycle.exchange_index`, `WsFill.exchange_index` (`i64`, required), `WsUserOrder.exchange_index`; added `ExchangeIndexStatus`/`GetExchangeStatusResponse.exchange_index_statuses`/`.intra_exchange_transfers_active` |
| New exchange-sharding endpoints: `POST/GET /portfolio/intra_exchange_instance_transfer(s)` (2026-08-13/20), `GET/POST /portfolio/target_balance_allocation` (2026-08-20), `DELETE /portfolio/events/orders` cancel-all (2026-08-27), `resting_order_value_breakdown` (2026-08-20) | Added `intra_exchange_instance_transfer`, `get_intra_exchange_instance_transfer(s)` + pager/stream, `get_target_balance_allocation`, `set_target_balance_allocation`, `cancel_all_orders_v2`, `GetPortfolioRestingOrderTotalValueResponse.resting_order_value_breakdown` (`IndexedBalance`) |
| Sub-account-restricted API keys (2026-07-02) | Added `subaccount` field to `CreateApiKeyRequest`/`GenerateApiKeyRequest` (other subaccount-restricted-key entries are permission-only, no schema) |
| RFQ-scoped quote actions + retention change (2026-06-25); RFQ-scoped quote lookup (2026-07-09) | Added `get_quote_by_rfq`, `delete_quote_by_rfq`, `accept_quote_by_rfq`, `confirm_quote_by_rfq`; deprecated the quote-ID-only equivalents |
| Quote time filters + pagination fix (2026-06-18) | Added `GetQuotesParams.min_ts`/`.max_ts` (the pagination fix is server-side) |
| Legacy `/portfolio/orders` mutation endpoints deprecated (2026-06-18) | Deprecated `create_order`, `cancel_order`, `amend_order`, `decrease_order`, `batch_create_orders`, `batch_cancel_orders` in favor of the `*_v2` event-order endpoints |
| Order group limit endpoint gains `subaccount`/`exchange_index` params (2026-08-06) | **Breaking** — `update_order_group_limit` now takes a `UpdateOrderGroupLimitParams` |
| Filter FCM orders by client order IDs (2026-09-03) | **Breaking** — `GetFcmOrdersParams.subtrader_id` is now `Option<String>`; added `.client_order_ids` |
| Historical positions endpoint + subaccount filter (2026-07-23, 2026-09-03) | Added `get_historical_positions` (reuses `GetPositionsParams`/`GetPositionsResponse`) |
| `settlement_sources` on events (2026-06-18) | Added `EventData.settlement_sources` (reuses `series::SettlementSource`) |
| `product_metadata.cadence` on events (2026-07-30) | Added `EventMetadata.cadence` |
| `tickers` filter on `GET /events` (2026-06-18) | Added `GetEventsParams.tickers` |
| `strike_type`/`cap_strike`/`custom_strike` on `metadata_updated`; `price_ranges` on `created`/`price_level_structure_updated` (2026-06-18, 2026-07-02) | Added top-level `strike_type`, `cap_strike`, `custom_strike`, `price_ranges` fields to `WsMarketLifecycleV2` (shared by `market_lifecycle_v2` and `multivariate_market_lifecycle`) |
| `is_block_trade` on WebSocket trade messages (2026-08-13) | Added `WsTrade.is_block_trade` (`bool`, `#[serde(default)]`, matching the existing REST `Trade.is_block_trade` pattern) |
| `subaccount` on `quote_created` WebSocket message, matching `quote_accepted`/`quote_executed` (2026-07-30) | Added `subaccount` to `WsQuoteCreated`, `WsQuoteAccepted`, `WsQuoteExecuted` — schema-drift fix: `rfq_creator_id` was also missing from `WsQuoteCreated`/`WsQuoteAccepted` and is now modeled |
| New endpoints: `GET /live_data/events/{event_ticker}` (2026-07-30), `GET /live_data/weather/{city}` (2026-08-20), `GET /live_data/weather/{city}/calibrations` (2026-08-31) | Added `get_event_live_data`, `get_weather_index`, `get_weather_index_calibrations` and their types |
| `GET /account/api_usage_level/volume_progress` (2026-06-11), `POST /account/api_usage_level/upgrade` (2026-06-11) | Added `get_account_api_usage_volume_progress`, `upgrade_account_api_usage_level` |
| `resting_margin_reservation` on target balance allocation (2026-09-03) | Added `SetTargetBalanceAllocationRequest.resting_margin_reservation` |
| `source_subaccount`/`destination_subaccount` on cross-shard transfers (2026-08-20) | Added to `IntraExchangeInstanceTransferRequest` |
| `cfbenchmarks_value_5hz` new WebSocket channel (2026-09-03); `pyth_value` new WebSocket channel (2026-07-23) | **Deferred** — both require new subscription-update actions (`SubscribeUnderlyings`/`UnsubscribeUnderlyings`/index-list variants), new error-code handling, and new message types. Not implemented this cycle; tracked in `docs/spec-parity.md` |
| Margin-exchange-only entries (asset_class, fee_tier_rates, exit triggers, order-group exchange_index binding, sided leverage estimates, order_reason, is_portfolio, margin_used/risk restrictions, perps mark prices/volume/OI, tick size, maker-volume incentive programs) | No code change — margin-exchange trading surfaces are not modeled by this crate |
| FIX-API-only entries (all `FIX`-tagged entries with no `REST`/`WebSocket` counterpart: market-data session limits, order identity, trade type, `ClearingBusinessDate`, RFQ combo-validation errors, quote identity, post-only quotes, exchange-index routing, entry timestamps, execution-report `LastMkt`, cancel/replace rejects, `AcceptQuote` reject reasons) | No code change — FIX is out of scope for this crate |
| FCM `GET /fcm/positions` filters, operational/permission/rate-limit/behavior-only notices (VPC peering, sharding notices, auto-routing default, `Accept-Language`, retention windows, tier-qualification changes, rate-limit-cost changes, sanity limits, order-group count limits, hidden-event filtering, amend `remaining_count` bugfix) | No code change |
| Margin fee-tier active rates (2026-06-11), fractional RFQ quantities (2026-06-11) | Already covered by the 0.6.0 disposition table (dated after the 0.6.0 watermark but reviewed then); no further action |

### Added

- [Rust API] `exchange_index` support across the exchange-sharding rollout (see disposition table): typed fields on `Market`, `Series`, `MultivariateEventCollection`, `Fill`, `Settlement`, `MarketPosition`, `SubaccountBalance`, `WsMarketLifecycleV2`, `WsEventLifecycle`, `WsFill`, `WsUserOrder`; `exchange_index` query filters on positions/fills/orders; `GetBalanceParams`/`get_balance_scoped`.
- [Rust API] New portfolio endpoints: `intra_exchange_instance_transfer`, `get_intra_exchange_instance_transfers` (+ pager/stream), `get_intra_exchange_instance_transfer`, `get_target_balance_allocation`, `set_target_balance_allocation`, `cancel_all_orders_v2`, `get_historical_positions`.
- [Rust API] New account endpoints: `get_account_api_usage_volume_progress`, `upgrade_account_api_usage_level`.
- [Rust API] New live-data endpoints: `get_event_live_data`, `get_weather_index`, `get_weather_index_calibrations`, with `EventLiveData`, `WeatherIndexPoint`, `WeatherIndexStationReading`, `WeatherIndexCalibration`, `WeatherIndexCalibrationStation` types.
- [Rust API] RFQ-scoped quote endpoints: `get_quote_by_rfq`, `delete_quote_by_rfq`, `accept_quote_by_rfq`, `confirm_quote_by_rfq`.
- [Rust API] `EventData.settlement_sources`, `EventMetadata.cadence`, `GetEventsParams.tickers`, `GetFcmOrdersParams.client_order_ids`, `GetQuotesParams.min_ts`/`.max_ts`/`.user_filter`, `CreateApiKeyRequest`/`GenerateApiKeyRequest.subaccount`.
- [Rust API] `GetPortfolioRestingOrderTotalValueResponse.resting_order_value_breakdown` (`IndexedBalance`).
- [Rust API] `WsMarketLifecycleV2` top-level `strike_type`, `cap_strike`, `custom_strike`, `price_ranges`; `WsTrade.is_block_trade`; `WsQuoteCreated`/`WsQuoteAccepted`/`WsQuoteExecuted.subaccount`; `WsQuoteCreated`/`WsQuoteAccepted.rfq_creator_id`.

### Changed

- [Rust API] `GetQuotesParams` no longer accepts `market_ticker`/`event_ticker` (removed upstream 2026-06-20); use `.user_filter`, `.rfq_id`, `.status`, or the new time filters instead.
- [Rust API] `GetFcmOrdersParams.subtrader_id` is now `Option<String>` (upstream now accepts `client_order_ids` as an alternative filter).
- [Rust API] `update_order_group_limit` takes a new `UpdateOrderGroupLimitParams` (`subaccount`, `exchange_index`) argument.

### Deprecated

- [Rust API] `create_order`, `cancel_order`, `amend_order`, `decrease_order`, `batch_create_orders`, `batch_cancel_orders` — use the `*_v2` event-order endpoints (legacy `/portfolio/orders` mutation endpoints were deprecated upstream 2026-06-18).
- [Rust API] `get_quote`, `delete_quote`, `accept_quote`, `confirm_quote` — use the RFQ-scoped equivalents (quote-ID-only actions were deprecated upstream 2026-06-25/07-09; quotes are no longer guaranteed queryable by ID alone).

### Removed

- [Rust API] `Market.response_price_units`, `Market.fractional_trading_enabled`, `MarketPosition.resting_orders_count`, `EventData.available_on_brokers` — removed from the upstream OpenAPI schema.
- [Rust API] `get_exchange_announcements`, `GetExchangeAnnouncementsResponse`, `Announcement`, `AnnouncementType`, `AnnouncementStatus` — `GET /exchange/announcements` was removed upstream.
- [Rust API] `lookup_tickers_for_market_in_multivariate_event_collection`, `get_multivariate_event_collection_lookup_history` and their request/response types — both multivariate lookup endpoints were removed upstream.
- [Rust API] The WebSocket `multivariate` channel: `WsChannelV2::Multivariate`, `WsMsgType::Multivariate`/`MultivariateLookup`, `WsMultivariate`, `WsMultivariateRef` — removed upstream; subscribing now returns an unknown-channel error.
- [Rust API] `WsMarketLifecycleEventType::FractionalTradingUpdated`, `WsMarketLifecycleV2.fractional_trading_enabled` (+ Ref) — removed from the upstream AsyncAPI schema alongside the REST field.
- [Rust API] Dead code: `ws::types::{MarketPositionRef, EventPositionRef}` — an unused, incorrect internal type that reused the REST `MarketPosition`/`EventPosition` shape for the `market_positions` WebSocket channel; the channel's actual wire shape has always been modeled separately as `WsMarketPosition`/`WsMarketPositionRef`.

### Fixed

- [Tests] A pre-existing test in `ws::types::envelope` did not account for the `sid`/`seq` fields added to `WsMessageV2::ListSubscriptions`/`WsMessageRef::ListSubscriptions` in 0.7.0; fixed the match patterns.

### Breaking

- [Rust API] `EventData`, `Market`, `MarketPosition` lose the fields listed under Removed. Downstream exhaustive struct destructuring must drop them; field-access sites must migrate off them (there is no replacement — Kalshi returns `false`/omits these permanently).
- [Rust API] `Fill`, `Settlement`, `MarketPosition`, `SubaccountBalance` gain a required `exchange_index: i64` field. Downstream struct-literal construction and exhaustive destructuring must account for it.
- [Rust API] `WsFill` (+ `WsFillRef`) gains a required `exchange_index: i64` field.
- [Rust API] The WebSocket `multivariate` channel is gone: `WsChannelV2`, `WsMsgType`, `WsDataMessageV2`, and `WsDataMessageRef` all lose their `Multivariate` variant. Exhaustive matches over these enums must drop the arm.
- [Rust API] `GetQuotesParams` loses `market_ticker`/`event_ticker`.
- [Rust API] `GetFcmOrdersParams.subtrader_id` changes from `String` to `Option<String>`.
- [Rust API] `update_order_group_limit(order_group_id, body)` becomes `update_order_group_limit(order_group_id, params, body)`.
- [Rust API] `lookup_tickers_for_market_in_multivariate_event_collection`, `get_multivariate_event_collection_lookup_history`, `get_exchange_announcements` and their types no longer exist.


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
