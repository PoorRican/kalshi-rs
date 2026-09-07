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

- The WebSocket error message's inner text field is wire-named `msg` (nested under the envelope's
  own `msg` object, i.e. `{"type":"error","msg":{"code":7,"msg":"..."}}`), not `message`. `WsError` /
  `WsErrorRef` use `#[serde(rename = "msg")]` on the Rust field `message` to reflect this — confirmed
  against the AsyncAPI `errorResponsePayload` schema (2026-09-10 docs correction; the wire behavior
  itself was not new). `sid`/`seq` on a subscription-scoped error are carried by the same top-level
  envelope fields used for every other message type, so no separate handling was needed once the
  field-name fix landed. Error codes 6, 16, and 17 are retired (never emitted, numbers reserved); the
  crate treats `WsError.code` as an opaque `Option<i64>` so no enum needed updating.

- The legacy `/portfolio/orders` mutation endpoints (`create_order`, `cancel_order`, `amend_order`,
  `decrease_order`, `batch_create_orders`, `batch_cancel_orders`) were deprecated 2026-06-18/25 and,
  as of the current OpenAPI spec, no longer appear in the documented surface at all — only
  `GET /portfolio/orders` and `GET /portfolio/orders/{order_id}` remain. Live calls to the mutation
  endpoints now return an error directing callers to the V2 event-order endpoints. The six Rust
  methods are marked `#[deprecated]` (pointing at their `*_v2` equivalents) rather than removed, since
  they still make syntactically valid requests and removing them would be a larger break than the
  upstream migration calls for. New code should use `create_order_v2` / `cancel_order_v2` /
  `amend_order_v2` / `decrease_order_v2` / `batch_create_orders_v2` / `batch_cancel_orders_v2`.

- `IncentiveProgram.incentive_description` is required upstream (non-`Option`); `target_size` (a
  legacy plain-integer field) is no longer in the schema and was removed — only `target_size_fp`
  remains. `max_reward_per_account` (`margin_maker_volume` programs only) is `Option<i64>`.

- `exchange_index` (`ExchangeIndex`, a plain `u32` shard identifier) was added across a wide swath of
  the upstream surface in 2026-07/08 as Kalshi rolled out exchange sharding. Every REST/WS type the
  crate already models that gained the field is `Option<u32>`, even where upstream marks it required,
  matching this crate's general defensive-parsing convention.

## Known Gaps (Upstream Surfaces Not Modeled)

These upstream REST endpoints and WebSocket channels are not implemented in the crate. They were
each introduced or changed by a changelog entry during the 2026-06-08 → 2026-09-10 refresh window,
but adding full support was judged out of scope for that pass. Listed so a future refresh doesn't
have to re-discover them:

- Margin exchange: markets, positions, risk, order groups, exit triggers, cancel-all, and most
  margin-specific fields are not modeled. Only `GET /margin/fee_tiers` is implemented.
- FIX API (order entry, market data, drop-copy) is entirely out of scope — this crate is REST/WebSocket only.
- New REST endpoints not yet wrapped: `GET /live_data/weather/{city}` and
  `.../calibrations`, `GET /live_data/events/{event_ticker}`, `GET /margin/fee_tier_rates`,
  `POST /portfolio/target_balance_allocation` (+ `GET`), `POST /portfolio/intra_exchange_instance_transfer`,
  `GET/POST /portfolio/intra_exchange_instance_transfers*`, cancel-all-orders endpoints,
  `GET /account/api_usage_level/volume_progress`, `POST /account/api_usage_level/upgrade`,
  RFQ-scoped quote action endpoints (`.../rfqs/{rfq_id}/quotes/{quote_id}/*`), the RFQ-scoped quote
  lookup endpoint (`GET .../rfqs/{rfq_id}/quotes/{quote_id}`).
- New WebSocket channels not yet wrapped: `pyth_value`, `cfbenchmarks_value_5hz`.
- `PUT /order_groups/{order_group_id}/limit` and multivariate event collection responses also gained
  `exchange_index`/`subaccount` fields not yet surfaced (order groups aren't otherwise exchange-index
  aware in this crate).

## Test Strategy

- Deterministic parsing and behavior checks: `tests/parsing.rs`,
  `tests/ws_parsing.rs`, `tests/ws_command_behavior.rs`
- Live contract checks: `tests/rest_public.rs`, `tests/rest_auth.rs`,
  `tests/ws_public.rs`, `tests/ws_auth.rs`
