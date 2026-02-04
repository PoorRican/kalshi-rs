# Kalshi REST API Reference

> ⚠️ **ALWAYS fetch current documentation** before implementing.
> This file provides pointers to official docs - verify all details at the source.

## Official Documentation

- **REST API Reference**: https://docs.kalshi.com/api-reference/exchange/get-exchange-status
- **OpenAPI Spec**: https://docs.kalshi.com/openapi.yaml
- **Quick Start (Authenticated)**: https://docs.kalshi.com/getting_started/quick_start_authenticated_requests
- **API Keys**: https://docs.kalshi.com/getting_started/api_keys

## Base URLs

```
Production: https://api.elections.kalshi.com/trade-api/v2
Demo:       https://demo-api.kalshi.co/trade-api/v2
```

## Authentication

**Docs**: https://docs.kalshi.com/getting_started/api_keys

Kalshi uses RSA-PSS signatures. Every authenticated request requires three headers:

| Header | Value |
|--------|-------|
| `KALSHI-ACCESS-KEY` | Your API Key ID |
| `KALSHI-ACCESS-TIMESTAMP` | Request timestamp in milliseconds |
| `KALSHI-ACCESS-SIGNATURE` | Base64-encoded RSA-PSS signature |

### Signature Generation

```
message = {timestamp_ms}{HTTP_METHOD}{path_without_query_params}
```

**CRITICAL**: Strip query parameters before signing!
- Sign `/trade-api/v2/portfolio/orders` NOT `/trade-api/v2/portfolio/orders?limit=5`

```python
# Example signature generation
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import padding
import base64

def sign_request(private_key, timestamp_ms, method, path):
    # MUST strip query params before signing
    path_without_query = path.split('?')[0]
    message = f"{timestamp_ms}{method}{path_without_query}".encode('utf-8')
    
    signature = private_key.sign(
        message,
        padding.PSS(
            mgf=padding.MGF1(hashes.SHA256()),
            salt_length=padding.PSS.DIGEST_LENGTH
        ),
        hashes.SHA256()
    )
    return base64.b64encode(signature).decode('utf-8')
```

## Rate Limits

**Docs**: https://docs.kalshi.com/getting_started/rate_limits

| Tier | Read (per sec) | Write (per sec) | Qualification |
|------|----------------|-----------------|---------------|
| Basic | 20 | 10 | Completing signup |
| Advanced | 30 | 30 | [Application form](https://kalshi.typeform.com/advanced-api) |
| Premier | 100 | 100 | 3.75% of exchange volume/month |
| Prime | 400 | 400 | 7.5% of exchange volume/month |

**Write-limited endpoints** (each batch item = 1 transaction, except BatchCancelOrders = 0.2):
- `POST /portfolio/orders` (CreateOrder)
- `DELETE /portfolio/orders/{order_id}` (CancelOrder)
- `POST /portfolio/orders/batched` (BatchCreateOrders)
- `DELETE /portfolio/orders/batched` (BatchCancelOrders)
- `POST /portfolio/orders/{order_id}/amend` (AmendOrder)
- `POST /portfolio/orders/{order_id}/decrease` (DecreaseOrder)

## Pagination

**Docs**: https://docs.kalshi.com/getting_started/pagination

Cursor-based pagination on list endpoints. Default limit: 100 (or 200 for events).

```python
def fetch_all_markets(series_ticker):
    all_markets = []
    cursor = None
    
    while True:
        params = {"series_ticker": series_ticker, "limit": 100}
        if cursor:
            params["cursor"] = cursor
            
        response = requests.get(f"{BASE_URL}/markets", params=params)
        data = response.json()
        
        all_markets.extend(data['markets'])
        cursor = data.get('cursor')
        
        if not cursor:
            break
    
    return all_markets
```

**Paginated endpoints**:
- `GET /markets`
- `GET /events`
- `GET /series`
- `GET /markets/trades`
- `GET /portfolio/orders`
- `GET /portfolio/fills`
- `GET /portfolio/positions`

## Key Endpoints

### Public (No Authentication)

| Endpoint | Description | Docs |
|----------|-------------|------|
| `GET /exchange/status` | Exchange status | [Link](https://docs.kalshi.com/api-reference/exchange/get-exchange-status) |
| `GET /markets` | List markets | [Link](https://docs.kalshi.com/api-reference/market/get-markets) |
| `GET /markets/{ticker}` | Single market | [Link](https://docs.kalshi.com/api-reference/market/get-market) |
| `GET /markets/{ticker}/orderbook` | Orderbook | [Link](https://docs.kalshi.com/api-reference/market/get-market-orderbook) |
| `GET /markets/trades` | Trade history | [Link](https://docs.kalshi.com/api-reference/market/get-trades) |
| `GET /events` | List events | [Link](https://docs.kalshi.com/api-reference/events/get-events) |
| `GET /events/multivariate` | Multivariate events | [Link](https://docs.kalshi.com/api-reference/events/get-multivariate-events) |
| `GET /series` | List series | [Link](https://docs.kalshi.com/api-reference/market/get-series-list) |
| `GET /candlesticks` | Event candlesticks | [Link](https://docs.kalshi.com/api-reference/events/get-event-candlesticks) |

### Authenticated (Requires RSA-PSS Signature)

| Endpoint | Description | Docs |
|----------|-------------|------|
| `GET /portfolio/balance` | Account balance | [Link](https://docs.kalshi.com/api-reference/portfolio/get-balance) |
| `GET /portfolio/positions` | Current positions | [Link](https://docs.kalshi.com/api-reference/portfolio/get-positions) |
| `GET /portfolio/orders` | List orders | [Link](https://docs.kalshi.com/api-reference/orders/get-orders) |
| `POST /portfolio/orders` | Create order | [Link](https://docs.kalshi.com/api-reference/orders/create-order) |
| `DELETE /portfolio/orders/{id}` | Cancel order | [Link](https://docs.kalshi.com/api-reference/orders/cancel-order) |
| `POST /portfolio/orders/batched` | Batch create | [Link](https://docs.kalshi.com/api-reference/orders/batch-create-orders) |
| `DELETE /portfolio/orders/batched` | Batch cancel | [Link](https://docs.kalshi.com/api-reference/orders/batch-cancel-orders) |
| `GET /portfolio/fills` | Trade fills | [Link](https://docs.kalshi.com/api-reference/portfolio/get-fills) |
| `GET /portfolio/settlements` | Settlements | [Link](https://docs.kalshi.com/api-reference/portfolio/get-settlements) |

## GET /markets Filtering

**Important constraints** (from changelog):

1. **Status filter**: Only ONE status filter allowed per request
2. **Timestamp filters are mutually exclusive**:

| Timestamp Filters | Compatible Status Filters |
|-------------------|---------------------------|
| `min_created_ts`, `max_created_ts` | `unopened`, `open`, or none |
| `min_close_ts`, `max_close_ts` | `closed`, or none |
| `min_settled_ts`, `max_settled_ts` | `settled`, or none |

3. **Multivariate filtering**: Use `mve_filter` parameter
   - `only` - Returns only multivariate events
   - `exclude` - Excludes multivariate events

## Order Creation

**Docs**: https://docs.kalshi.com/api-reference/orders/create-order

```json
{
  "ticker": "MARKET-TICKER",
  "side": "yes",           // or "no"
  "action": "buy",         // or "sell"
  "count": 10,             // number of contracts
  "type": "limit",         // only "limit" supported now
  "yes_price": 50,         // price in cents (or use yes_price_dollars)
  "time_in_force": "gtc",  // gtc, ioc, or fok
  "client_order_id": "optional-unique-id"
}
```

**Important changes** (verify in changelog):
- `order_type` field no longer required, only `limit` supported
- Cannot specify both `time_in_force: "ioc"` AND `expiration_ts`
- Past `expiration_ts` values now rejected (previously converted to IoC)

## Response Price Formats

**Docs**: https://docs.kalshi.com/getting_started/subpenny_pricing

Two formats are returned:

```json
{
  "yes_bid": 56,                    // Legacy: cents (integer)
  "yes_bid_dollars": "0.5600"       // New: fixed-point dollars (string)
}
```

**Prepare for migration**: Legacy integer cent fields will be deprecated. Use `_dollars` fields.

## Fixed-Point Contract Quantities

**Docs**: https://docs.kalshi.com/getting_started/fixed_point_contracts

```json
{
  "count": 10,           // Legacy: integer
  "count_fp": "10.00"    // New: fixed-point (string, 0-2 decimals)
}
```

Currently must be whole numbers. Fractional trading coming per-market.

## Error Handling

Common HTTP status codes:

| Code | Meaning |
|------|---------|
| 400 | Invalid request (check error message) |
| 401 | Authentication failed |
| 403 | Forbidden (insufficient permissions) |
| 404 | Resource not found |
| 429 | Rate limited |
| 500/503/504 | Server error |

Always check the response body for detailed error messages.
