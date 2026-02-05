/// Example of using an authenticated WS channel
///
/// This channel is explicitly called out in the docs as being authenticated
use kalshi::{
    KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannel, WsEvent, WsReconnectConfig,
    WsSubscriptionParams,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = KalshiEnvironment::demo();

    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;

    let mut ws =
        KalshiWsClient::connect_authenticated(env, auth, WsReconnectConfig::default()).await?;

    ws.subscribe(WsSubscriptionParams {
        channels: vec![WsChannel::OrderbookDelta],
        market_tickers: Some(vec!["SOME_MARKET_TICKER".to_string()]),
        ..Default::default()
    })
    .await?;

    loop {
        match ws.next_event().await? {
            WsEvent::Message(msg) => println!("{:?}", msg),
            WsEvent::Reconnected { attempt } => println!("Reconnected (attempt {})", attempt),
            WsEvent::Disconnected { error } => {
                println!("Disconnected: {:?}", error);
                break;
            }
        }
    }

    Ok(())
}
