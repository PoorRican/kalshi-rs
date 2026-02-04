# Kalshi WebSocket API Reference

> ⚠️ **ALWAYS fetch current documentation** before implementing.
> This file provides pointers to official docs - verify all details at the source.

## Official Documentation

- **WebSocket Connection**: https://docs.kalshi.com/websockets/websocket-connection
- **Connection Keep-Alive**: https://docs.kalshi.com/websockets/connection-keep-alive
- **Orderbook Updates**: https://docs.kalshi.com/websockets/orderbook-updates
- **Market Ticker**: https://docs.kalshi.com/websockets/market-ticker
- **Market Ticker V2**: https://docs.kalshi.com/websockets/market-ticker-v2
- **Public Trades**: https://docs.kalshi.com/websockets/public-trades
- **User Fills**: https://docs.kalshi.com/websockets/user-fills
- **Market Positions**: https://docs.kalshi.com/websockets/market-positions
- **Market & Event Lifecycle**: https://docs.kalshi.com/websockets/market-&-event-lifecycle

## WebSocket URL

```
Production: wss://api.elections.kalshi.com
Demo:       wss://demo-api.kalshi.co/trade-api/v2/ws
```

## Authentication

**All WebSocket connections require authentication**, even for public market data channels.

Include API key headers during the WebSocket handshake:
- `KALSHI-ACCESS-KEY`
- `KALSHI-ACCESS-TIMESTAMP`
- `KALSHI-ACCESS-SIGNATURE`

## Connection Keep-Alive (CRITICAL)

**Docs**: https://docs.kalshi.com/websockets/connection-keep-alive

Kalshi sends **Ping frames (0x9)** every **10 seconds** with body `heartbeat`.

**You MUST respond with Pong frames (0xA)** or the connection will be closed.

```python
# Most WebSocket libraries handle this automatically
# If implementing manually:

# Kalshi sends: Ping frame (0x9) with body "heartbeat"
# You respond: Pong frame (0xA)
```

### Connection Limits

WebSocket connections per user are limited by API tier:
- Default limit: 200 connections
- Increases with higher API usage tiers

## Commands

### Subscribe

```json
{
  "id": 1,
  "cmd": "subscribe",
  "params": {
    "channels": ["orderbook_delta"],
    "market_ticker": "MARKET-TICKER"
  }
}
```

### Unsubscribe

```json
{
  "id": 2,
  "cmd": "unsubscribe",
  "params": {
    "sids": [1, 2]
  }
}
```

### List Subscriptions

```json
{
  "id": 3,
  "cmd": "list_subscriptions"
}
```

### Update Subscription (Add Markets)

```json
{
  "id": 4,
  "cmd": "update_subscription",
  "params": {
    "sids": [456],
    "market_tickers": ["NEW-MARKET-1", "NEW-MARKET-2"],
    "action": "add_markets"
  }
}
```

### Update Subscription (Remove Markets)

```json
{
  "id": 5,
  "cmd": "update_subscription",
  "params": {
    "sids": [456],
    "market_tickers": ["MARKET-TO-REMOVE"],
    "action": "delete_markets"
  }
}
```

## Response Types

### Subscribed

```json
{
  "id": 1,
  "type": "subscribed",
  "msg": {
    "channel": "orderbook_delta",
    "sid": 1
  }
}
```

### Unsubscribed

```json
{
  "id": 2,
  "sid": 2,
  "seq": 7,
  "type": "unsubscribed"
}
```

### OK (for update operations)

```json
{
  "id": 3,
  "sid": 456,
  "seq": 222,
  "type": "ok",
  "market_tickers": ["MARKET-1", "MARKET-2"]
}
```

### Error

```json
{
  "id": 123,
  "type": "error",
  "msg": {
    "code": 6,
    "msg": "Already subscribed"
  }
}
```

## Channels

| Channel | Description | Auth Required | Docs |
|---------|-------------|---------------|------|
| `orderbook_delta` | Orderbook updates | Yes (conn) | [Link](https://docs.kalshi.com/websockets/orderbook-updates) |
| `ticker` | Market price updates | Yes (conn) | [Link](https://docs.kalshi.com/websockets/market-ticker) |
| `ticker_v2` | Enhanced ticker | Yes (conn) | [Link](https://docs.kalshi.com/websockets/market-ticker-v2) |
| `trade` | Public trade feed | Yes (conn) | [Link](https://docs.kalshi.com/websockets/public-trades) |
| `fill` | User's trade fills | Yes (conn+data) | [Link](https://docs.kalshi.com/websockets/user-fills) |
| `market_position` | Position updates | Yes (conn+data) | [Link](https://docs.kalshi.com/websockets/market-positions) |
| `market_lifecycle` | Market status changes | Yes (conn) | [Link](https://docs.kalshi.com/websockets/market-&-event-lifecycle) |

## Subscription Behavior

**Repeated subscriptions no longer error** (as of Sep 25, 2025):
- Same market tickers: No action taken
- New market tickers: Added to existing subscription

Use `list_subscriptions` command to view active subscriptions.

## Subpenny Price Fields

WebSocket messages now include both formats:

```json
{
  "yes_bid": 56,
  "yes_bid_dollars": "0.5600"
}
```

## Example: Python WebSocket Client

```python
import asyncio
import websockets
import json

async def connect_kalshi(api_key_id, private_key, market_ticker):
    url = "wss://api.elections.kalshi.com"

    # Generate auth headers (see REST.md for signature generation)
    timestamp_ms = str(int(time.time() * 1000))
    signature = sign_request(private_key, timestamp_ms, "GET", "/trade-api/ws/v2")

    headers = {
        "KALSHI-ACCESS-KEY": api_key_id,
        "KALSHI-ACCESS-TIMESTAMP": timestamp_ms,
        "KALSHI-ACCESS-SIGNATURE": signature
    }

    async with websockets.connect(url, additional_headers=headers) as ws:
        # Subscribe to orderbook updates
        await ws.send(json.dumps({
            "id": 1,
            "cmd": "subscribe",
            "params": {
                "channels": ["orderbook_delta"],
                "market_ticker": market_ticker
            }
        }))

        # Listen for messages
        async for message in ws:
            data = json.loads(message)
            print(data)

# Note: Most WebSocket libraries handle ping/pong automatically
# The websockets library responds to pings by default
```

## Important Notes

1. **Single connection endpoint**: All communication through one WebSocket connection
2. **Authentication required**: Even for public data channels
3. **Ping/Pong handling**: Usually automatic, but verify your library handles it
4. **Sequence numbers**: Messages include `seq` for ordering; use to detect gaps
5. **`client_order_id` in orderbook_delta**: Appears when YOUR order caused the change
