# Spec Notes

This repository follows Kalshi's published OpenAPI and AsyncAPI documents
directly.

Those upstream specs are the baseline for contract review, but they do not
fully define every public behavior in the crate. The most important behavior
checks live in tests, especially where the YAML specs are underspecified or
examples are ambiguous.

## Known Distinctions

- `MarketStatusQuery` is the query/filter enum used by list endpoints.
- `MarketStatus` is the lifecycle/status enum returned on market objects.
- They overlap, but they are not one-to-one. Lifecycle states such as
  `determined`, `disputed`, and `amended` collapse differently when converted
  into query status. The conversion behavior is covered in `tests/parsing.rs`.

- The AsyncAPI examples imply both singular and plural market ticker fields for
  websocket subscriptions.
- The crate accepts `market_ticker` or `market_tickers`, but not both.
- `orderbook_delta` requires market tickers and rejects `market_id` and
  `market_ids`.
- `skip_ticker_ack` is supported on subscription updates.
- These behaviors are covered by `tests/ws_command_behavior.rs` and
  `tests/ws_parsing.rs`.

- The AsyncAPI spec marks `ts_ms` as required on both the `trade` and
  `ticker` channel messages (`WsTrade`, `WsTicker`).
- In practice the field is occasionally omitted by the exchange. Consumers
  should treat `ts_ms` as best-effort and fall back to `ts` (seconds) when
  precise millisecond timing matters.

- The `side` and `action` fields on `Order`, `Fill`, and `WsFill` were deprecated by Kalshi on
  2026-05-07. The new normalized fields are `outcome_side` (`yes` | `no`) and `book_side`
  (`bid` | `ask`), where `bid` ≡ `yes` and `ask` ≡ `no`. The OpenAPI/AsyncAPI specs still mark the
  legacy fields required ("not removed before May 14, 2026"), but the changelog scheduled removal
  for 2026-05-28. To survive either state, the legacy fields are modeled as `Option`, and the new
  normalized fields are also `Option` so older payloads (lacking them) still parse.
- The public `Trade` object (REST `Trade`, WebSocket `WsTrade`) uses the taker-prefixed variants:
  `taker_side` (deprecated) plus `taker_outcome_side` / `taker_book_side`. These follow the same
  `Option` treatment for the same reasons.

- The `/margin/fee_tiers` response was restructured on 2026-05-11. The previous tier-name maps
  (`maker_fee_tiers`, `taker_fee_tiers`) were replaced by per-ticker decimal-rate maps
  (`maker_fee_rates`, `taker_fee_rates`). Fee is computed as `notional * rate`.

- `event_fee_update` is an AsyncAPI message delivered on the `market_lifecycle_v2` channel (it is
  not a separately-subscribable channel). It is modeled by `WsEventFeeUpdate`. `fee_type_override`
  is kept as `Option<String>` rather than reusing the `FeeType` enum so the raw string survives any
  future fee-type additions without a crate update. Both override fields are nullable (`None` when
  the override is cleared).

- `FeeType` enum now includes `QuadraticWithMakerFees` (serialized `quadratic_with_maker_fees`),
  added to the OpenAPI spec in 2026. An `#[serde(other)] Unknown` catch-all is also present so
  unrecognised future variants never panic during deserialization. `fee_type_override` on
  `WsEventFeeUpdate` remains `Option<String>` for lossless round-trip regardless.

- `is_block_trade: bool` was added to the public REST `Trade` struct (2026-05-29). The field is
  `#[serde(default)]` (defaults to `false`) so payloads predating the flag still parse. The query
  filter `GetTradesParams::is_block_trade: Option<bool>` lets callers filter by block-trade status.

- `GET /account/limits` (`get_account_api_limits`) response was restructured in 2026-06 (automated
  API rate-limit tiers). The old flat shape (`read_limit: i64, write_limit: i64`) was replaced by
  nested `BucketLimit` objects (`read: BucketLimit, write: BucketLimit`) plus a `grants:
  Vec<ApiUsageLevelGrant>` array. The `GetAccountApiLimitsResponse` struct was updated accordingly;
  old field access will not compile (intentional minor-version break, 0.5.0 → 0.6.0).
  `ApiUsageLevelGrant.expires_ts` is `Option<i64>` because the field is absent for non-expiring
  grants.

- `cfbenchmarks_value` is a new AsyncAPI channel (introduced 2026-06) that delivers CF Benchmarks
  index values. It uses `index_ids` (not market tickers) for subscription parameters; pass
  `["all"]` to receive all available indices. The channel emits two message types:
  `cfbenchmarks_value` (per-index value + 60-second windowed average) and
  `cfbenchmarks_value_indexlist` (the full set of available index IDs). Both are modeled as
  `WsCfBenchmarksValue` / `WsCfBenchmarksIndexList` and routed through the standard
  `WsDataMessageV2` enum. `last_60s_windowed_average_15min` on `WsCfBenchmarksValue` is `Option`
  because the spec marks it conditional. The documented post-subscribe workflow (discover indices
  via `indexlist`, then add/remove with `subscribe_indices` / `unsubscribe_indices`) is supported
  through `update_subscription_v2` using the `WsUpdateAction::SubscribeIndices` /
  `UnsubscribeIndices` / `Indexlist` actions plus the `index_ids` field on
  `WsUpdateSubscriptionParamsV2`. `validate_update` rejects mixing index actions with market targets
  and requires `index_ids` for the add/remove actions, matching the AsyncAPI error semantics.

- `GET /account/endpoint_costs` (`get_account_endpoint_costs`) is modeled as a public (unauthed)
  endpoint because the OpenAPI operation declares no `security` requirement, unlike `/account/limits`.
  `ApiUsageLevelGrant.exchange_instance` is kept as `String` rather than an `ExchangeInstance` enum
  (`event_contract` | `margined`); the raw string round-trips losslessly and tolerates any future
  exchange-instance values without a crate update.
- The AsyncAPI marks several timestamp/required fields that the exchange may omit in practice
  (`ts_ms` on ticker/trade/order-group messages, the legacy direction fields). These are modeled as
  `Option` so parsing never fails on their absence.

- **Exchange sharding (2026 rollout).** Kalshi is provisioning dedicated exchange shards
  (`exchange_index`) for select market categories. `exchange_index` is `i64` (not `Option`) on
  `Fill`, `Settlement`, `MarketPosition`, `SubaccountBalance`, and the WebSocket `WsFill` message,
  matching the OpenAPI/AsyncAPI `required` list for those exact shapes. It is `Option<i64>` on
  `Market`, `Series`, `MultivariateEventCollection`, `WsMarketLifecycleV2` (only present on
  `created` events), `WsEventLifecycle`, and `WsUserOrder`, matching the spec there. Only 0 is
  currently live in production; the field exists to future-proof clients ahead of sharding.
- `ws::types::mod.rs` previously carried an internal `MarketPositionRef`/`EventPositionRef` pair
  that reused the REST `MarketPosition`/`EventPosition` shapes for the `market_positions` WebSocket
  channel. This was dead code — the channel's real wire shape (`user_id`, `market_ticker`,
  `position_cost_dollars`, `position_fee_cost_dollars`, `volume_fp`, ...) has always been modeled
  correctly and separately as `WsMarketPosition`/`WsMarketPositionRef` in
  `ws/types/messages/positions.rs`, which is what's actually wired into the parser. Removed in
  0.8.0 rather than patched, since it was never reachable.
- `WsQuoteCreated` and `WsQuoteAccepted` were missing `rfq_creator_id` (present in the AsyncAPI
  schema since at least 0.6.0) and all three quote lifecycle messages (`WsQuoteCreated`,
  `WsQuoteAccepted`, `WsQuoteExecuted`) were missing `subaccount` (added upstream 2026-07-30, and
  documented as "matching `quote_accepted`/`quote_executed`" even though neither actually had it in
  this crate). All four fields are now modeled as `Option`, since none of these three structs carry
  a flatten `extra` catch-all.
- `pyth_value` (AsyncAPI, added 2026-07-23) and `cfbenchmarks_value_5hz` (added 2026-09-03) are new
  WebSocket channels **not yet implemented**. Both need new `WsUpdateAction` variants
  (`SubscribeUnderlyings`/`UnsubscribeUnderlyings`/an underlying-list action for `pyth_value`; an
  index-list variant shared with `cfbenchmarks_value` for the 5Hz channel), new subscription
  validation (mirroring the `index_ids` handling already in place for `cfbenchmarks_value`), and new
  message types (`pythValue`, `pythUnderlyingList`, `cfbenchmarksValue5Hz`,
  `cfbenchmarks5HzIndexList`). Deferred to a follow-up refresh rather than adding a partial,
  unvalidated implementation.
- `Market.price_level_structure` stays an opaque `String` (not an enum) by design: the exchange has
  added many new structure values over time (`center_deci_edge_centi_cent` and seven
  `center_{center}_edge_{edge}_cent` variants as of 2026-08), and upstream explicitly recommends
  reading valid prices from `price_ranges` dynamically rather than keying logic off the structure
  name. The same reasoning applies to the top-level `price_level_structure` field on
  `WsMarketLifecycleV2`.
- `FractionalTradingUpdated` was removed from `WsMarketLifecycleEventType` and
  `fractional_trading_enabled` was removed from `WsMarketLifecycleV2` alongside the REST
  `Market.fractional_trading_enabled` removal (2026-07-09): the AsyncAPI `market_lifecycle_v2`
  `event_type` enum no longer lists `fractional_trading_updated`, and the field is gone from the
  payload schema.

## Test Strategy

- Deterministic parsing and behavior checks: `tests/parsing.rs`,
  `tests/ws_parsing.rs`, `tests/ws_command_behavior.rs`
- Live contract checks: `tests/rest_public.rs`, `tests/rest_auth.rs`,
  `tests/ws_public.rs`, `tests/ws_auth.rs`
