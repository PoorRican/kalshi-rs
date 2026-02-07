# Kalshi Rust Client

## WebSocket Auth
Kalshi WebSocket connections require authentication, even when subscribing to public channels. Use `KalshiWsClient::connect_authenticated` and provide `KALSHI_KEY_ID` and `KALSHI_PRIVATE_KEY_PATH`.

## Environment Variables
The examples load environment variables via `dotenvy`. Set these in your shell or a `.env` file:
- `KALSHI_KEY_ID`
- `KALSHI_PRIVATE_KEY_PATH`

Integration tests load `.env.test`. Create a `.env.test` file with the same variables if you want to run authenticated tests.

## Pagination & Truncation
There are two pagination styles:

- **Pager (page-level)**: `CursorPager<T>` yields full pages (`Vec<T>`). You control when to stop by **not** calling `next_page()` again. This is ideal when you want to checkpoint the cursor between calls.
- **Stream (item-level)**: `stream_*` yields items one-by-one. You can stop early with `.take(n)` or by providing `max_items`, and it won’t fetch extra pages once that limit is hit.

Examples:

Page-level:
```rust
let mut pager = client.markets_pager(GetMarketsParams::default());
while let Some(page) = pager.next_page().await? {
    for market in page {
        println!("{}", market.ticker);
    }
}
```

Item-level:
```rust
use futures::stream::TryStreamExt;

let markets: Vec<_> = client
    .stream_markets(GetMarketsParams::default(), Some(250))
    .try_collect()
    .await?;
```

Note: `get_*_all` loads **all pages into memory** and is intended for convenience only.

## WebSocket Reconnect
`KalshiWsClient` is the high-level WebSocket client with auto reconnect + resubscribe. It emits explicit connection events:

- `WsEvent::Message(...)`
- `WsEvent::Raw(...)` (when using `WsReaderMode::Raw`)
- `WsEvent::Reconnected { attempt }`
- `WsEvent::Disconnected { error }`

Reconnect uses exponential backoff with jitter, resubscribes to active channels by default, and **does not** attempt sequence resync (callers must handle any gaps).
