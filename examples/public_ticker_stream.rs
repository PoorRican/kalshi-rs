use anyhow;
use kalshi::{
    KalshiAuth, KalshiEnvironment, KalshiWsClient, WsChannel, WsDataMessage, WsMessage,
    WsSubscriptionParams,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env = KalshiEnvironment::demo();
    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;

    let mut ws = KalshiWsClient::connect_authenticated(env, auth).await?;

    ws.subscribe(WsSubscriptionParams {
        channels: vec![WsChannel::Ticker],
        ..Default::default()
    })
    .await?;

    loop {
        let msg = ws.next_message().await?;
        match msg {
            WsMessage::Data(WsDataMessage::Ticker { msg, .. }) => {
                println!(
                    "type=ticker market={} price={}",
                    msg.market_ticker, msg.price
                );
            }
            other => {
                println!("type=other msg={:?}", other);
            }
        }
    }
}
