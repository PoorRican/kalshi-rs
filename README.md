# kalshi-fast-rs

[![Crates.io](https://img.shields.io/crates/v/kalshi-fast-rs.svg)](https://crates.io/crates/kalshi-fast-rs)
[![Documentation](https://docs.rs/kalshi-fast-rs/badge.svg)](https://docs.rs/kalshi-fast-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance async Rust client for the [Kalshi](https://kalshi.com) prediction markets API.

## Features

- **Full WebSocket support** - Real-time streaming with auto-reconnect and resubscribe
- **Complete REST API** - All public and authenticated endpoints
- **Pagination helpers** - Page-level (`CursorPager`) and item-level (`stream_*`) iteration
- **RSA-PSS authentication** - Secure signing for private endpoints

## Installation

```sh
cargo add kalshi-fast-rs
```

## Quick Start

See [examples](https://github.com/PoorRican/kalshi-fast-rs/tree/master/examples) for more advanced usage.

### REST API - List Markets

```rust
use kalshi_fast::{GetMarketsParams, KalshiEnvironment, KalshiRestClient, MarketStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = KalshiRestClient::new(KalshiEnvironment::demo());

    let resp = client
        .get_markets(GetMarketsParams {
            limit: Some(10),
            status: Some(MarketStatus::Open),
            ..Default::default()
        })
        .await?;

    for market in resp.markets {
        println!("{}", market.ticker);
    }
    Ok(())
}
```

### WebSocket - Ticker Stream

```rust
use kalshi_fast::{
    KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannel,
    WsDataMessage, WsEvent, WsMessage, WsReconnectConfig, WsSubscriptionParams,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;

    let mut ws = KalshiWsClient::connect_authenticated(
        KalshiEnvironment::demo(),
        auth,
        WsReconnectConfig::default(),
    ).await?;

    ws.subscribe(WsSubscriptionParams {
        channels: vec![WsChannel::Ticker],
        ..Default::default()
    }).await?;

    loop {
        match ws.next_event().await? {
            WsEvent::Message(WsMessage::Data(WsDataMessage::Ticker { msg, .. })) => {
                println!("{}: {}", msg.market_ticker, msg.price);
            }
            WsEvent::Reconnected { attempt } => println!("Reconnected (attempt {})", attempt),
            WsEvent::Disconnected { .. } => break,
            _ => {}
        }
    }
    Ok(())
}
```

## Performance

Optimized for low-latency algorithmic trading:

- **Deferred JSON parsing** - Uses `serde_json::RawValue` to skip parsing unused fields
- **Zero-copy message parsing** - Binary WebSocket frames parsed with `from_slice`
- **Split read/write streams** - No lock contention on WebSocket operations
- **Efficient subscription tracking** - HashMap-based channel management

## Pagination

Two pagination styles are available:

**Page-level** with `CursorPager`:
```rust
let mut pager = client.markets_pager(GetMarketsParams::default());
while let Some(page) = pager.next_page().await? {
    for market in page {
        println!("{}", market.ticker);
    }
}
```

**Item-level** with streams:
```rust
use futures::stream::TryStreamExt;

let markets: Vec<_> = client
    .stream_markets(GetMarketsParams::default(), Some(250))
    .try_collect()
    .await?;
```

## WebSocket Reconnect

`KalshiWsClient` handles reconnection automatically with exponential backoff and resubscribes to active channels. Connection events are exposed via `WsEvent`:

- `WsEvent::Message(...)` - Incoming data
- `WsEvent::Reconnected { attempt }` - Connection restored
- `WsEvent::Disconnected { error }` - Connection lost (after max retries)

Note: Sequence resync is not automatic; callers must handle any gaps.

## Environment Variables

For authenticated endpoints:
- `KALSHI_KEY_ID` - Your API key ID
- `KALSHI_PRIVATE_KEY_PATH` - Path to your RSA private key (PEM format)

## Documentation

- [API Documentation](https://docs.rs/kalshi-fast-rs)
- [Examples](https://github.com/PoorRican/kalshi-fast-rs/tree/master/examples)
- [Kalshi API Docs](https://trading-api.readme.io/reference/getting-started)

## License

MIT
