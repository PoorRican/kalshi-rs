/// Example of using an authenticated WS channel
///
/// This channel is explicitly called out in the docs as being authenticated

use kalshi::{KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannel, WsSubscriptionParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = KalshiEnvironment::demo();

    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;

    let mut ws = KalshiWsClient::connect_authenticated(env, auth).await?;

    ws.subscribe(WsSubscriptionParams {
        channels: vec![WsChannel::OrderbookDelta],
        market_tickers: Some(vec!["SOME_MARKET_TICKER".to_string()]),
        ..Default::default()
    })
    .await?;

    loop {
        let msg = ws.next_message().await?;
        println!("{:?}", msg);
    }
}
