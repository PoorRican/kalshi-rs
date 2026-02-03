use anyhow;
use kalshi::{KalshiEnvironment, KalshiWsClient, WsChannel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = KalshiEnvironment::demo();
    let mut ws = KalshiWsClient::connect(env).await?;

    ws.subscribe(vec![WsChannel::Ticker], None).await?;

    loop {
        let msg = ws.next_envelope().await?;
        println!("type={} id={:?} msg={:?}", msg.msg_type, msg.id, msg.msg);
    }
}

