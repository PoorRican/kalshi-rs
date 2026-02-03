/// Example of using authenticated REST endpoints:
/// - Gets balance
/// - Places an order

use kalshi_adapter::{KalshiEnvironment, KalshiRestClient};
use kalshi_adapter::rest::{GetMarketsParams, MarketStatus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = KalshiEnvironment::demo();
    let client = KalshiRestClient::new(env);

    let resp = client
        .get_markets(GetMarketsParams {
            limit: Some(1),
            status: Some(MarketStatus::Open),
            ..Default::default()
        })
        .await?;

    println!("markets: {}", resp.markets.len());
    Ok(())
}

