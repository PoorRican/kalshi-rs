# Kalshi Rust Client

## WebSocket Auth
Kalshi WebSocket connections require authentication, even when subscribing to public channels. Use `KalshiWsClient::connect_authenticated` and provide `KALSHI_KEY_ID` and `KALSHI_PRIVATE_KEY_PATH`.

## Environment Variables
The examples load environment variables via `dotenvy`. Set these in your shell or a `.env` file:
- `KALSHI_KEY_ID`
- `KALSHI_PRIVATE_KEY_PATH`

Integration tests load `.env.test`. Create a `.env.test` file with the same variables if you want to run authenticated tests.
