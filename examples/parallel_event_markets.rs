/// Fetch markets for multiple events concurrently.
///
/// Demonstrates parallel stream usage with try_join_all for efficient
/// multi-event market lookups.

use futures::future::try_join_all;
use futures::stream::TryStreamExt;
use kalshi::{GetMarketsParams, KalshiEnvironment, KalshiRestClient, Market};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = KalshiRestClient::new(KalshiEnvironment::production());
    let event_tickers = vec!["INXD-25FEB14", "INXD-25FEB21"]; // example tickers

    let futures: Vec<_> = event_tickers
        .iter()
        .map(|ticker| {
            let client = client.clone();
            let ticker = ticker.to_string();
            async move {
                let markets: Vec<Market> = client
                    .stream_markets(
                        GetMarketsParams {
                            event_ticker: Some(vec![ticker.clone()]),
                            ..Default::default()
                        },
                        None,
                    )
                    .try_collect()
                    .await?;
                Ok::<_, kalshi::KalshiError>((ticker, markets))
            }
        })
        .collect();

    for (ticker, markets) in try_join_all(futures).await? {
        println!("{}: {} markets", ticker, markets.len());
    }

    Ok(())
}
