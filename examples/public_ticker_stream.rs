use anyhow;
use kalshi_fast::{
    KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannel, WsDataMessage, WsEvent, WsMessage,
    WsReaderConfig, WsReconnectConfig, WsSubscriptionParams,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env = KalshiEnvironment::demo();
    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;

    let mut ws =
        KalshiWsClient::connect_authenticated(env, auth, WsReconnectConfig::default()).await?;

    ws.subscribe(WsSubscriptionParams {
        channels: vec![WsChannel::Ticker],
        ..Default::default()
    })
    .await?;

    let events = ws.start_reader(WsReaderConfig::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            WsEvent::Message(msg) => match msg {
                WsMessage::Data(WsDataMessage::Ticker { msg, .. }) => {
                    println!(
                        "type=ticker market={} price={}",
                        msg.market_ticker, msg.price
                    );
                }
                other => {
                    println!("type=other msg={:?}", other);
                }
            },
            WsEvent::Raw(_) => {}
            WsEvent::Reconnected { attempt } => {
                println!("type=reconnected attempt={}", attempt);
            }
            WsEvent::Disconnected { error } => {
                println!("type=disconnected error={:?}", error);
                break;
            }
        }
    }

    Ok(())
}
