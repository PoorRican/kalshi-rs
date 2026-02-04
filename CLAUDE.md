# CLAUDE.md

## Purpose

Rust adapter for the Kalshi Exchange API designed for **algorithmic trading**. Must be fully tested and feature complete.

## Performance Requirements

This adapter must be as performant as possible. Every design decision should consider latency and efficiency:

- Minimize allocations in hot paths
- Avoid unnecessary cloning - prefer borrowing
- No runtime overhead from abstractions
- WebSocket message parsing must be zero-copy where possible
- Pre-compute anything that can be pre-computed (e.g., auth signatures reuse the same signing key)

## Architecture

- `auth.rs` - RSA-PSS SHA256 signing. Message format: `{timestamp_ms}{METHOD}{path_without_query}`
- `rest.rs` - HTTP client via reqwest. Public endpoints need no auth, `/portfolio/*` endpoints require signed headers
- `ws.rs` - WebSocket via tokio-tungstenite. Split read/write streams for concurrent operations. Private channels (OrderbookDelta, Fill, etc.) require authenticated connection
- `types.rs` - Request/response types. Uses `serde_json::Value` for market data responses to decouple from schema changes
- `env.rs` - Demo vs Production base URLs

## Examples

- `list_open_markets.rs` - Public REST: fetching market data
- `place_order.rs` - Authenticated REST: order placement
- `public_ticker_stream.rs` - Public WebSocket: ticker subscription
- `lob_delta.rs` - Authenticated WebSocket: order book deltas (requires `KALSHI_KEY_ID` and `KALSHI_PRIVATE_KEY_PATH` env vars)
