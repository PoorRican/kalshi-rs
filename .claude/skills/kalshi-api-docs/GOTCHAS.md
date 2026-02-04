# Kalshi API Gotchas & Critical Information

> ⚠️ **This file contains critical surprises and common pitfalls.**
> Always verify against the official changelog: https://docs.kalshi.com/changelog

## Binary Market Mechanics

### Orderbook Only Returns BIDS (Not Asks!)

**Docs**: https://docs.kalshi.com/getting_started/orderbook_responses

This is **counter-intuitive** but critical to understand:

```json
{
  "orderbook": {
    "yes": [[42, 13], [41, 10]],  // YES bids only
    "no": [[56, 17], [45, 20]]    // NO bids only
  }
}
```

**Why?** In binary markets, there's a reciprocal relationship:
- **YES bid at X¢** = **NO ask at (100-X)¢**
- **NO bid at Y¢** = **YES ask at (100-Y)¢**

**To calculate spreads:**
```python
best_yes_bid = orderbook['yes'][-1][0]  # Last = highest (sorted ascending)
best_no_bid = orderbook['no'][-1][0]

best_yes_ask = 100 - best_no_bid   # Implied from NO side
best_no_ask = 100 - best_yes_bid   # Implied from YES side

yes_spread = best_yes_ask - best_yes_bid
```

### Arrays Are Sorted Ascending

Orderbook arrays are sorted by price in **ascending order**.
- **Best bid (highest)** = **LAST element** in array
- **Worst bid (lowest)** = **FIRST element** in array

## Authentication Pitfalls

### Strip Query Params Before Signing

**WRONG:**
```python
path = "/trade-api/v2/portfolio/orders?limit=5"
message = f"{timestamp}GET{path}"  # ❌ Includes query params
```

**CORRECT:**
```python
path = "/trade-api/v2/portfolio/orders?limit=5"
path_without_query = path.split('?')[0]  # ✓
message = f"{timestamp}GET{path_without_query}"
```

### Private Key Is Never Retrievable

When you generate an API key, the private key is shown **ONCE**.
- Download it immediately
- Store securely
- Cannot be retrieved again from Kalshi

## Breaking Changes (Recent)

**Always check**: https://docs.kalshi.com/changelog

### IoC Orders Cannot Have Expiration (Nov 2025)

```json
// ❌ NOW REJECTED
{
  "time_in_force": "immediate_or_cancel",
  "expiration_ts": 1234567890
}

// ✓ CORRECT - Pick one
{
  "time_in_force": "immediate_or_cancel"
}
// OR
{
  "expiration_ts": 1234567890  // Will not be IoC
}
```

### Past Expiration Timestamps Now Rejected (Nov 2025)

Previously: Past `expiration_ts` → Auto-converted to IoC
Now: **Rejected with error** "Expiration timestamp must be in the future"

### GET /events Excludes Multivariate Events (Dec 2025)

```python
# ❌ Won't return multivariate events
requests.get("/events")

# ✓ Use dedicated endpoint for multivariate
requests.get("/events/multivariate")
```

### 'Pending' Status Removed from Orders (Nov 2025)

Order status enum no longer includes `pending`.

### Timestamp Filters Are Mutually Exclusive (Nov 2025)

In `GET /markets`, you cannot combine certain timestamp filters:

| These filters... | ...only work with these statuses |
|------------------|----------------------------------|
| `min_created_ts`, `max_created_ts` | `unopened`, `open`, or none |
| `min_close_ts`, `max_close_ts` | `closed`, or none |
| `min_settled_ts`, `max_settled_ts` | `settled`, or none |

### Only ONE Status Filter Allowed (Nov 2025)

```python
# ❌ REJECTED
params = {"status": "open,closed"}

# ✓ CORRECT - One status only
params = {"status": "open"}
```

## Pricing Migration

### Subpenny Pricing Coming

**Docs**: https://docs.kalshi.com/getting_started/subpenny_pricing

Two formats now returned:
```json
{
  "price": 56,                  // Legacy: integer cents
  "price_dollars": "0.5600"     // New: fixed-point string
}
```

**Action Required**: Migrate to `_dollars` fields. Legacy cents will be deprecated.

### Fixed-Point Contracts Coming

**Docs**: https://docs.kalshi.com/getting_started/fixed_point_contracts

```json
{
  "count": 10,           // Legacy: integer
  "count_fp": "10.00"    // New: fixed-point string
}
```

Currently `_fp` must be whole numbers. Fractional trading coming per-market.

## Rate Limit Surprises

### Batch Cancel Is Cheaper

- Regular operations: 1 transaction each
- **BatchCancelOrders**: Each cancel = **0.2 transactions**

Use batch cancel to maximize throughput!

### Tier Requirements

- Basic → Advanced: Requires [application form](https://kalshi.typeform.com/advanced-api)
- Premier/Prime: Requires volume targets AND technical competency review

## WebSocket Gotchas

### Must Handle Ping/Pong

Kalshi sends **Ping (0x9)** every **10 seconds** with body `"heartbeat"`.
- You must respond with **Pong (0xA)**
- Most libraries handle this automatically
- Verify your implementation!

### Connection Requires Auth Even for Public Data

Even public channels like `orderbook_delta` require authenticated connection.

### Repeated Subscriptions Don't Error

As of Sep 2025:
- Same tickers: No action taken
- New tickers: Added to existing subscription

## URL Confusion

### Multiple Production URLs Exist

**Current Production**: `https://api.elections.kalshi.com`

You may see references to:
- `https://trading-api.kalshi.com` (older)
- `https://api.kalshi.com` (older)

**Always verify current URL in docs.**

### Demo Environment Completely Separate

- Demo URL: `https://demo-api.kalshi.co`
- Demo Web: `https://demo.kalshi.co`
- **Credentials are NOT shared** between demo and production

## Order Type Changes

### Only Limit Orders Supported

`order_type` is no longer required. Only `limit` type is supported.

```json
// ✓ Price required for all orders now
{
  "yes_price": 50,
  "side": "yes"
}
```

### Self-Trade Prevention Behavior

When FoK orders self-cross:
- `taker_at_cross`: Taker canceled, partial fills executed
- `maker`: Maker canceled, execution continues, remaining taker canceled

## Data Format Quirks

### Tags Parameter Parsing Changed (Oct 2025)

`GET /series?tags=...` now splits on **commas only**, not spaces.

```
# ✓ WORKS: Comma-separated
?tags=Rotten Tomatoes,Television

# Parsed as: ["Rotten Tomatoes", "Television"]
```

### Trailing Slashes Behavior Changed

- Without slash: Returns 200 ✓
- With slash: Returns 301 redirect to version without slash

## Missing Features/Deprecations

### GetApiVersion Removed

API versioning endpoint no longer exists (as of July 2025).

### resting_orders_count Removed

No longer returned in `GET /portfolio/positions` (Nov 2025).

## Testing Recommendations

1. **Always start with demo environment**
2. **Subscribe to changelog RSS**: `https://docs.kalshi.com/changelog.rss`
3. **Check pending API spec**: Use "version" dropdown in docs
4. **Join Discord #dev channel** for support

## Quick Sanity Checks

Before going live, verify:
- [ ] Signature strips query params
- [ ] Using current production URL
- [ ] Handling ping/pong for WebSocket
- [ ] Using `_dollars` fields where available
- [ ] Not combining incompatible filters
- [ ] Understanding orderbook only shows bids
