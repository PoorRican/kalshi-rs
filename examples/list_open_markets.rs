/// Example of using the Public REST endpoints: lists open markets

use kalshi::{KalshiEnvironment, KalshiRestClient};
use kalshi::rest::{GetMarketsParams, MarketStatus};

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

