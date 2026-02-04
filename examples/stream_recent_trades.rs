/// Stream trades using TryStreamExt adapters with time-based filtering.
///
/// Demonstrates the stream API with max_items limit for collecting a subset
/// of results.

use chrono::{Duration, Utc};
use futures::stream::TryStreamExt;
use kalshi::{GetTradesParams, KalshiEnvironment, KalshiRestClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = KalshiRestClient::new(KalshiEnvironment::production());
    let cutoff_ts = (Utc::now() - Duration::hours(24)).timestamp();

    let trades: Vec<_> = client
        .stream_trades(
            GetTradesParams {
                min_ts: Some(cutoff_ts),
                limit: Some(100),
                ..Default::default()
            },
            Some(500), // max 500 trades
        )
        .try_collect()
        .await?;

    println!("Found {} trades in last 24h", trades.len());
    Ok(())
}
