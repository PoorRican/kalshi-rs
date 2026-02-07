/// Example: consume raw WS events and parse a borrowed view
use anyhow;
use kalshi_fast::{
    KalshiEnvironment, KalshiWsClient, WsChannel, WsDataMessageRef, WsEvent, WsMessageRef,
    WsReaderConfig, WsReaderMode, WsReconnectConfig, WsSubscriptionParams,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = KalshiEnvironment::demo();
    let mut ws = KalshiWsClient::connect(env, WsReconnectConfig::default()).await?;

    ws.subscribe(WsSubscriptionParams {
        channels: vec![WsChannel::Ticker],
        ..Default::default()
    })
    .await?;

    let events = ws
        .start_reader(WsReaderConfig {
            buffer_size: 1024,
            mode: WsReaderMode::Raw,
        })
        .await?;

    while let Some(event) = events.next().await {
        match event {
            WsEvent::Raw(raw) => {
                let msg = raw.parse_borrowed()?;
                if let WsMessageRef::Data(WsDataMessageRef::Ticker { msg, .. }) = msg {
                    println!(
                        "type=ticker market={} price={}",
                        msg.market_ticker, msg.price
                    );
                }
            }
            WsEvent::Reconnected { attempt } => println!("Reconnected (attempt {})", attempt),
            WsEvent::Disconnected { error } => {
                println!("Disconnected: {:?}", error);
                break;
            }
            WsEvent::Message(_) => {}
        }
    }

    Ok(())
}
