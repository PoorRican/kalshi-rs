/// Example: Find a high-volume event and stream orderbook deltas for all its markets
///
/// 1. Queries markets to find one with 24h volume > threshold
/// 2. Fetches all markets for that event
/// 3. Subscribes to orderbook deltas for all markets in the event
/// 4. Prints each delta update via debug logging
///
/// Requires KALSHI_KEY_ID and KALSHI_PRIVATE_KEY_PATH env vars (or .env file)
use kalshi::{
    GetMarketsParams, KalshiAuth, KalshiEnvironment, KalshiRestClient, KalshiWsClient, Market,
    MarketStatus, MveFilter, WsDataMessage, WsEvent, WsMessage, WsReconnectConfig,
    WsSubscriptionParams,
};
use std::time::Duration;
use tokio::time::sleep;

const MIN_VOLUME_24H: i64 = 10_000;
const MAX_PAGES: usize = 50;

fn get_volume(market: &Market) -> i64 {
    market.volume_24h.unwrap_or(0)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env = KalshiEnvironment::production();
    let client = KalshiRestClient::new(env.clone());

    // Step 1: Paginate through markets to find one with high volume
    println!("Searching for markets with 24h volume > {MIN_VOLUME_24H}...");

    let mut cursor: Option<String> = None;
    let mut target_market: Option<Market> = None;

    for _page in 1..=MAX_PAGES {
        let resp = client
            .get_markets(GetMarketsParams {
                limit: Some(100),
                status: Some(MarketStatus::Open),
                mve_filter: Some(MveFilter::Exclude),
                cursor: cursor.clone(),
                ..Default::default()
            })
            .await?;

        print!(".");

        if let Some(m) = resp
            .markets
            .into_iter()
            .find(|m| get_volume(m) > MIN_VOLUME_24H)
        {
            target_market = Some(m);
            break;
        }

        match resp.cursor {
            Some(c) if !c.is_empty() => cursor = Some(c),
            _ => break,
        }

        // Rate limit: 100ms between requests (extra safety; client also rate limits)
        sleep(Duration::from_millis(100)).await;
    }

    println!();

    let target_market = match target_market {
        Some(m) => m,
        None => {
            anyhow::bail!("No market found with 24h volume > {MIN_VOLUME_24H}");
        }
    };

    let event_ticker = target_market
        .event_ticker
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Market missing event_ticker"))?;

    println!(
        "Found event: {} (volume: {})",
        event_ticker,
        get_volume(&target_market)
    );

    // Step 2: Fetch all markets for this event
    let event_markets = client
        .get_markets(GetMarketsParams {
            limit: Some(100),
            event_ticker: Some(vec![event_ticker.to_string()]),
            status: Some(MarketStatus::Open),
            ..Default::default()
        })
        .await?;

    let market_tickers: Vec<String> = event_markets
        .markets
        .iter()
        .map(|m| m.ticker.clone())
        .collect();

    println!(
        "Subscribing to {} markets: {:?}",
        market_tickers.len(),
        market_tickers
    );

    // Step 3: Connect authenticated WebSocket
    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;

    let mut ws =
        KalshiWsClient::connect_authenticated(env, auth, WsReconnectConfig::default()).await?;

    // Step 4: Subscribe to orderbook deltas
    let sub_id = ws
        .subscribe(WsSubscriptionParams {
            channels: vec![kalshi::WsChannel::OrderbookDelta],
            market_tickers: Some(market_tickers),
            ..Default::default()
        })
        .await?;

    println!("Subscribed (id={}), streaming...\n", sub_id);

    // Step 5: Stream updates
    loop {
        match ws.next_event().await? {
            WsEvent::Message(msg) => match msg {
                WsMessage::Data(WsDataMessage::OrderbookSnapshot { msg, seq, .. }) => {
                    println!(
                        "[SNAPSHOT] {} | yes={} no={} | seq={:?}",
                        msg.market_ticker,
                        msg.yes.len(),
                        msg.no.len(),
                        seq
                    );
                }
                WsMessage::Data(WsDataMessage::OrderbookDelta { msg, seq, .. }) => {
                    println!(
                        "[DELTA] {} | {}@{} {:+} | seq={:?}",
                        msg.market_ticker, msg.side, msg.price, msg.delta, seq
                    );
                }
                WsMessage::Subscribed { sid, .. } => println!("[SUBSCRIBED] sid={:?}", sid),
                WsMessage::Error { error, .. } => println!("[ERROR] {:?}", error),
                other => println!("[OTHER] {:?}", other),
            },
            WsEvent::Reconnected { attempt } => println!("[RECONNECTED] attempt={}", attempt),
            WsEvent::Disconnected { error } => {
                println!("[DISCONNECTED] {:?}", error);
                break;
            }
        }
    }

    Ok(())
}
