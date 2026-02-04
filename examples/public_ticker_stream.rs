use anyhow;
use kalshi::{KalshiEnvironment, KalshiWsClient, WsDataMessage, WsMessage, WsSubscriptionParams, WsChannel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = KalshiEnvironment::demo();
    let mut ws = KalshiWsClient::connect(env).await?;

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
