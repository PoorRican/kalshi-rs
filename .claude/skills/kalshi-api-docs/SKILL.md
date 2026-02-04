---
name: kalshi-api-docs
description: "Reference for Kalshi prediction markets API. Use this skill when working with the Kalshi API adapter codebase. Points to official documentation for terminology, endpoints, parameters, API changes, and implementation details. ALWAYS fetch current docs - never rely on internal knowledge."
---

# Kalshi API Documentation Reference

> ⚠️ **CRITICAL**: Always fetch and reference the official Kalshi documentation directly.
> **NEVER rely on internal/training knowledge** for Kalshi API details - the API changes frequently.
> Use `web_fetch` to retrieve current documentation before implementing any API interactions.

## Official Documentation URLs

| Resource | URL |
|----------|-----|
| **Glossary** | https://docs.kalshi.com/getting_started/terms |
| **Rate Limits** | https://docs.kalshi.com/getting_started/rate_limits |
| **Pagination** | https://docs.kalshi.com/getting_started/pagination |
| **Orderbook Responses** | https://docs.kalshi.com/getting_started/orderbook_responses |
| **Subpenny Pricing** | https://docs.kalshi.com/getting_started/subpenny_pricing |
| **Fixed-Point Contracts** | https://docs.kalshi.com/getting_started/fixed_point_contracts |
| **REST API Reference** | https://docs.kalshi.com/api-reference/exchange/get-exchange-status |
| **WebSocket Reference** | https://docs.kalshi.com/websockets/websocket-connection |
| **WebSocket Keep-Alive** | https://docs.kalshi.com/websockets/connection-keep-alive |
| **API Changelog** | https://docs.kalshi.com/changelog |
| **OpenAPI Spec** | https://docs.kalshi.com/openapi.yaml |

> **Note**: Return type schemas are available in the HTML versions of the documentation pages, not the markdown variants.

## Base URLs

| Environment | REST API | WebSocket |
|-------------|----------|-----------|
| **Production** | `https://api.elections.kalshi.com/trade-api/v2` | `wss://api.elections.kalshi.com` |
| **Demo** | `https://demo-api.kalshi.co/trade-api/v2` | `wss://demo-api.kalshi.co/trade-api/v2/ws` |

## Core Terminology

These definitions are from the official glossary. Always verify at https://docs.kalshi.com/getting_started/terms

### Hierarchy: Series → Event → Market

- **Series**: Collection of related events with the same ticker prefix. Events in a series look at similar data over disjoint time periods with no logical outcome dependency between them.

- **Event**: Collection of markets; the basic unit members interact with. Each event belongs to a series.

- **Market**: A single binary market. The lowest-level tradeable object. Markets resolve to YES ($1.00) or NO ($0.00).

### Ticker Format

```
{SERIES_TICKER}-{EVENT_IDENTIFIER}-{MARKET_IDENTIFIER}
Example: KXHIGHNY-24JAN01-T60
         │         │        └── Market: Temperature > 60°F
         │         └── Event: January 1, 2024
         └── Series: NYC High Temperature
```

## Market Statuses

| Status | Description |
|--------|-------------|
| `unopened` | Market created but not yet open for trading |
| `open` | Market is active and accepting orders |
| `paused` | Trading temporarily suspended |
| `closed` | Trading ended, awaiting settlement |
| `settled` | Final result determined, payouts complete |

## Contract Model

- Binary contracts: YES and NO sides
- Prices in cents (1-99) with `_dollars` fields for subpenny precision
- **YES price + NO price = 100¢** (always)
- Settlement: YES pays $1.00, NO pays $0.00
- A YES bid at X¢ is equivalent to a NO ask at (100-X)¢

## Detailed References

See the following files for implementation details:

- @REST.md - REST API endpoints, authentication, rate limits
- @WEBSOCKET.md - WebSocket connection, channels, keep-alive
- @GOTCHAS.md - Critical surprises, breaking changes, common pitfalls

## Before You Code

1. **Fetch the changelog**: https://docs.kalshi.com/changelog
2. **Check for breaking changes**: API evolves rapidly
3. **Use demo environment first**: https://demo.kalshi.co
4. **Subscribe to RSS**: https://docs.kalshi.com/changelog.rss
